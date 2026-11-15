//! Captura multipágina, visor, tira editable y exportación sin sobrescribir archivos.
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use iced::widget::{
    button, column, container, image as imagen, mouse_area, pick_list, row, scrollable, text,
};
use iced::{Element, Length};

use super::{
    Mensaje,
    tema::Tokens,
    widgets::{boton_primario, boton_secundario, color},
};
use crate::nucleo::{
    escaneo::{self, FormatoSalida, Modo, OpcionesEscaneo},
    pdfsal::{self, PaginaEscaneada},
};

#[derive(Debug, Clone)]
pub struct Captura {
    pub pagina: PaginaEscaneada,
    pub visor: imagen::Handle,
}

pub fn capturar(
    dispositivo: String,
    opciones: OpcionesEscaneo,
    cancelar: Arc<AtomicBool>,
) -> Result<Captura, String> {
    let binario = std::env::var("PRINTDESK_SCANIMAGE").unwrap_or_else(|_| "scanimage".into());
    let pagina = escaneo::escanear(&dispositivo, &opciones, cancelar, &binario)
        .map_err(|e| format!("{e:?}"))?;
    let visor = image::load_from_memory(&pagina.png)
        .map_err(|e| e.to_string())?
        .thumbnail(1000, 1400)
        .to_rgba8();
    Ok(Captura {
        pagina,
        visor: imagen::Handle::from_rgba(visor.width(), visor.height(), visor.into_raw()),
    })
}

#[derive(Debug)]
pub struct EstadoEscanear {
    pub opciones: OpcionesEscaneo,
    pub paginas: Vec<Captura>,
    pub activa: usize,
    pub arrastrada: Option<usize>,
    pub sobre: Option<usize>,
    pub escaneando: bool,
    pub cancel: Arc<AtomicBool>,
    pub guardando: bool,
    pub id: u64,
    pub aviso: Option<String>,
    pub ruta_guardada: Option<PathBuf>,
}

impl Default for EstadoEscanear {
    fn default() -> Self {
        Self {
            opciones: OpcionesEscaneo::predeterminadas(),
            paginas: vec![],
            activa: 0,
            arrastrada: None,
            sobre: None,
            escaneando: false,
            guardando: false,
            cancel: Arc::new(AtomicBool::new(false)),
            id: 0,
            aviso: None,
            ruta_guardada: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MensajeEscanear {
    Escanear,
    Lista(Result<Captura, String>),
    Cancelar,
    Girar(usize),
    Eliminar(usize),
    Seleccionar(usize),
    Mover(usize, usize),
    Arrastrar(usize),
    Sobre(usize),
    Soltar,
    Resolucion(u32),
    Modo(Modo),
    Formato(FormatoSalida),
    Guardar,
    Guardado(Result<PathBuf, String>),
    AbrirCarpeta,
}

impl std::fmt::Display for Modo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Color => "Color",
            Self::Grises => "Gris",
            Self::Linea => "Línea",
        })
    }
}
impl std::fmt::Display for FormatoSalida {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Pdf => "PDF único",
            Self::Png => "PNG por página",
            Self::Jpg => "JPG por página",
        })
    }
}

pub fn carpeta() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("Escaneos")
}

pub fn guardar(
    paginas: &[PaginaEscaneada],
    formato: FormatoSalida,
    dpi: u16,
    dir: &Path,
    nombre: &str,
) -> Result<PathBuf, String> {
    if paginas.is_empty() || dpi == 0 {
        return Err("No hay páginas para guardar".into());
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    if formato == FormatoSalida::Pdf {
        let tmp = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
        pdfsal::paginas_a_pdf(paginas, tmp.path(), dpi).map_err(|e| format!("{e:?}"))?;
        let mut pendiente = tmp;
        for n in 0..1000 {
            let nombre = if n == 0 {
                format!("{nombre}.pdf")
            } else {
                format!("{nombre}-{n}.pdf")
            };
            let destino = dir.join(nombre);
            match pendiente.persist_noclobber(&destino) {
                Ok(_) => return Ok(destino),
                Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => pendiente = e.file,
                Err(e) => return Err(e.error.to_string()),
            }
        }
        return Err("Demasiados archivos con este nombre".into());
    }
    let carpeta = tempfile::Builder::new()
        .prefix(&format!("{nombre}-"))
        .tempdir_in(dir)
        .map_err(|e| e.to_string())?;
    let ext = if formato == FormatoSalida::Png {
        "png"
    } else {
        "jpg"
    };
    for (n, p) in paginas.iter().enumerate() {
        let imagen = image::load_from_memory(&p.png).map_err(|e| e.to_string())?;
        let imagen = match p.rotacion % 360 {
            90 => imagen.rotate90(),
            180 => imagen.rotate180(),
            270 => imagen.rotate270(),
            _ => imagen,
        };
        imagen
            .to_rgb8()
            .save(carpeta.path().join(format!("pagina-{}.{ext}", n + 1)))
            .map_err(|e| e.to_string())?;
    }
    Ok(carpeta.keep().join(format!("pagina-1.{ext}")))
}

pub fn update(e: &mut EstadoEscanear, m: MensajeEscanear) {
    if e.guardando && !matches!(m, MensajeEscanear::Guardado(_)) {
        return;
    }
    match m {
        MensajeEscanear::Escanear if !e.escaneando => {
            e.escaneando = true;
            e.id += 1;
            e.cancel = Arc::new(AtomicBool::new(false));
            e.aviso = None;
        }
        MensajeEscanear::Lista(res) => {
            e.escaneando = false;
            match res {
                Ok(p) => {
                    e.paginas.push(p);
                    e.activa = e.paginas.len() - 1;
                }
                Err(error) => e.aviso = Some(error),
            }
        }
        MensajeEscanear::Cancelar => e.cancel.store(true, Ordering::SeqCst),
        MensajeEscanear::Seleccionar(i) if i < e.paginas.len() => e.activa = i,
        MensajeEscanear::Girar(i) => {
            if let Some(p) = e.paginas.get_mut(i) {
                p.pagina.rotacion = (p.pagina.rotacion + 90) % 360;
            }
        }
        MensajeEscanear::Eliminar(i) if i < e.paginas.len() => {
            e.paginas.remove(i);
            e.activa = e.activa.min(e.paginas.len().saturating_sub(1));
        }
        MensajeEscanear::Mover(a, b) if a < e.paginas.len() && b < e.paginas.len() => {
            let pagina = e.paginas.remove(a);
            e.paginas.insert(b, pagina);
            e.activa = b;
        }
        MensajeEscanear::Arrastrar(i) => {
            e.arrastrada = Some(i);
            e.sobre = Some(i);
        }
        MensajeEscanear::Sobre(i) => e.sobre = Some(i),
        MensajeEscanear::Soltar => {
            if let (Some(a), Some(b)) = (e.arrastrada.take(), e.sobre.take()) {
                update(e, MensajeEscanear::Mover(a, b));
            }
        }
        // Una sesión conserva sus DPI; así ninguna hoja cambia de tamaño al guardar.
        MensajeEscanear::Resolucion(r) if e.paginas.is_empty() && !e.escaneando => {
            e.opciones.resolucion = r
        }
        MensajeEscanear::Modo(m) if !e.escaneando => e.opciones.modo = m,
        MensajeEscanear::Formato(f) if !e.escaneando => e.opciones.formato_salida = f,
        MensajeEscanear::Guardar if !e.paginas.is_empty() && !e.escaneando => {
            e.guardando = true;
            e.id += 1;
        }
        MensajeEscanear::Guardado(res) => {
            e.guardando = false;
            match res {
                Ok(ruta) => {
                    e.aviso = Some("Guardado. Las páginas siguen disponibles para revisar.".into());
                    e.ruta_guardada = Some(ruta);
                }
                Err(error) => e.aviso = Some(error),
            }
        }
        MensajeEscanear::AbrirCarpeta => {
            if let Err(error) = std::process::Command::new("xdg-open")
                .arg(carpeta())
                .spawn()
            {
                e.aviso = Some(error.to_string());
            }
        }
        _ => {}
    }
}

pub fn vista<'a>(t: &Tokens, e: &'a EstadoEscanear) -> Element<'a, Mensaje> {
    let m = Mensaje::Escanear;
    let controls = row![
        column![
            text("Resolución"),
            pick_list([150, 300, 600], Some(e.opciones.resolucion), move |r| m(
                MensajeEscanear::Resolucion(r)
            ))
        ],
        column![
            text("Color"),
            pick_list(
                [Modo::Color, Modo::Grises, Modo::Linea],
                Some(e.opciones.modo),
                move |v| m(MensajeEscanear::Modo(v))
            )
        ],
        column![
            text("Salida"),
            pick_list(
                [FormatoSalida::Pdf, FormatoSalida::Png, FormatoSalida::Jpg],
                Some(e.opciones.formato_salida),
                move |v| m(MensajeEscanear::Formato(v))
            )
        ],
        if e.escaneando {
            boton_secundario(t, "Cancelar escaneo", m(MensajeEscanear::Cancelar))
        } else {
            boton_primario(t, "Escanear página", m(MensajeEscanear::Escanear))
        },
    ]
    .spacing(12)
    .align_y(iced::Alignment::End);
    let mut tira = row![].spacing(8);
    for (i, p) in e.paginas.iter().enumerate() {
        tira = tira.push(
            mouse_area(
                column![
                    button(
                        imagen(&p.visor)
                            .width(72)
                            .height(96)
                            .rotation(iced::Radians((p.pagina.rotacion as f32).to_radians()))
                    )
                    .on_press(m(MensajeEscanear::Seleccionar(i))),
                    text(format!("Página {}", i + 1)).size(11),
                    row![
                        button("Girar").on_press(m(MensajeEscanear::Girar(i))),
                        button("Quitar").on_press(m(MensajeEscanear::Eliminar(i)))
                    ]
                    .spacing(3),
                    row![
                        button("←").on_press_maybe(
                            (i > 0).then(|| m(MensajeEscanear::Mover(i, i.saturating_sub(1))))
                        ),
                        button("→").on_press_maybe(
                            (i + 1 < e.paginas.len()).then(|| m(MensajeEscanear::Mover(i, i + 1)))
                        )
                    ]
                    .spacing(3)
                ]
                .spacing(4),
            )
            .on_press(m(MensajeEscanear::Arrastrar(i)))
            .on_enter(m(MensajeEscanear::Sobre(i)))
            .on_release(m(MensajeEscanear::Soltar)),
        );
    }
    let preview: Element<'a, Mensaje> = if let Some(p) = e.paginas.get(e.activa) {
        imagen(&p.visor)
            .rotation(iced::Radians((p.pagina.rotacion as f32).to_radians()))
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    } else {
        text("Coloca una hoja A4 en el cristal y pulsa Escanear página.").into()
    };
    let lienzo = color(t.lienzo);
    let preview = container(preview)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16)
        .style(move |_| container::Style {
            background: Some(lienzo.into()),
            border: iced::border::rounded(12),
            ..Default::default()
        });
    let guardar_btn = button(if e.guardando {
        "Guardando…"
    } else {
        "Guardar"
    })
    .on_press_maybe(
        (!e.paginas.is_empty() && !e.escaneando && !e.guardando)
            .then(|| m(MensajeEscanear::Guardar)),
    );
    column![
        controls,
        preview,
        scrollable(tira)
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default()
            ))
            .height(180),
        row![
            guardar_btn,
            boton_secundario(t, "Abrir carpeta", m(MensajeEscanear::AbrirCarpeta)),
            text(format!("{} página(s)", e.paginas.len()))
        ]
        .spacing(12),
        text(e.aviso.clone().unwrap_or_else(|| {
            e.ruta_guardada
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        }))
        .size(12),
    ]
    .spacing(12)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn captura(id: u8) -> Captura {
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbImage::from_pixel(10, 14, image::Rgb([id, 0, 0]))
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        Captura {
            pagina: PaginaEscaneada {
                png: png.into_inner(),
                rotacion: 0,
            },
            visor: imagen::Handle::from_rgba(1, 1, vec![id, 0, 0, 255]),
        }
    }
    #[test]
    fn edita_paginas_en_orden() {
        let mut e = EstadoEscanear::default();
        for id in 1..=3 {
            update(&mut e, MensajeEscanear::Lista(Ok(captura(id))));
        }
        let primera = e.paginas[0].pagina.png.clone();
        update(&mut e, MensajeEscanear::Mover(0, 2));
        assert_eq!(e.paginas[2].pagina.png, primera);
        update(&mut e, MensajeEscanear::Girar(2));
        assert_eq!(e.paginas[2].pagina.rotacion, 90);
        update(&mut e, MensajeEscanear::Eliminar(1));
        assert_eq!(e.paginas.len(), 2);
    }
    #[test]
    fn guardar_no_sobrescribe_y_preserva_paginas_en_pantalla() {
        let dir = tempfile::tempdir().unwrap();
        let paginas = vec![captura(1).pagina, captura(2).pagina];
        let a = guardar(&paginas, FormatoSalida::Pdf, 300, dir.path(), "escaneo").unwrap();
        let b = guardar(&paginas, FormatoSalida::Pdf, 300, dir.path(), "escaneo").unwrap();
        assert_ne!(a, b);
        for p in [a, b] {
            let pdf = exprim::RenderSession::open(std::fs::read(p).unwrap().into(), None).unwrap();
            assert_eq!(pdf.page_count(), 2);
        }
        assert!(guardar(&[], FormatoSalida::Pdf, 300, dir.path(), "vacio").is_err());
    }
    #[test]
    fn imagenes_guardadas_tienen_la_ruta_real() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = guardar(
            &[captura(1).pagina],
            FormatoSalida::Jpg,
            300,
            dir.path(),
            "imagen",
        )
        .unwrap();
        assert!(ruta.exists());
        assert!(image::open(ruta).is_ok());
    }
}

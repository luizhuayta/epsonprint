//! Pantalla Imprimir: cargar documento, vista previa fiel por hoja con zoom,
//! opciones completas y envío real a CUPS.

use std::path::PathBuf;

use iced::widget::{
    button, column, container, image, pick_list, row, scrollable, text, text_input,
};
use iced::{Element, Length};

use crate::nucleo::impresion::{self, Calidad, ColorImp, Escala, Margenes, Orientacion, Papel};
use crate::nucleo::pdfsal;
use crate::nucleo::trabajos::{TipoTrabajo, Trabajo};

use super::tema::Tokens;
use super::widgets::{boton_primario, boton_secundario, color};
use crate::ui::Mensaje;

#[derive(Debug, Clone)]
pub struct Documento {
    pub nombre: String,
    pub bytes: Vec<u8>,
    pub paginas: u32,
    pub ruta: PathBuf,
}

#[derive(Debug, Clone)]
pub struct EstadoImprimir {
    pub documento: Option<Documento>,
    pub opciones: impresion::OpcionesImpresion,
    pub hojas: Vec<image::Handle>,
    pub hoja_activa: usize,
    pub previsualizando: bool,
    pub id_preview: u64,
    pub abriendo: bool,
    pub id_abrir: u64,
    pub ruta_abrir: Option<PathBuf>,
    pub zoom: f32,
    pub ver_miniaturas: bool,
    pub ver_opciones: bool,
    pub texto_rango: String,
    pub error_rango: Option<String>,
    pub imprimiendo: bool,
    pub id_impresion: u64,
    pub aviso: Option<String>,
    pub porcentaje: String,
}

impl Default for EstadoImprimir {
    fn default() -> Self {
        Self {
            documento: None,
            opciones: impresion::OpcionesImpresion::predeterminadas(),
            hojas: Vec::new(),
            hoja_activa: 0,
            previsualizando: false,
            id_preview: 0,
            abriendo: false,
            id_abrir: 0,
            ruta_abrir: None,
            zoom: 0.75,
            ver_miniaturas: true,
            ver_opciones: true,
            texto_rango: "Todas".into(),
            error_rango: None,
            imprimiendo: false,
            id_impresion: 0,
            aviso: None,
            porcentaje: "100".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MensajeImprimir {
    Abrir,
    DocumentoCargado(Result<Documento, String>),
    CambiarPapel(Papel),
    CambiarOrientacion(Orientacion),
    CambiarColor(ColorImp),
    CambiarCalidad(Calidad),
    CambiarMargenes(Margenes),
    CambiarEscala(EscalaPicked),
    PorcentajeEditado(String),
    CambiarCopias(DeltaCopias),
    RangoEditado(String),
    RangoEnviado,
    HojasPrevisualizadas(u64, Result<Vec<Vec<u8>>, String>),
    HojaActiva(usize),
    Zoom(DeltaZoom),
    AlternarMiniaturas,
    AlternarOpciones,
    Imprimir,
    Impreso(Result<String, String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaCopias {
    Mas,
    Menos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaZoom {
    Mas,
    Menos,
    Ajustar,
}

/// Lo que el selector de escala ofrece; el % fijo se traduce a `Escala`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscalaPicked {
    Ajustar,
    Real,
    CientoVeinte,
    Ochenta,
    Cincuenta,
}

impl EscalaPicked {
    pub const TODAS: [Self; 5] = [
        Self::Ajustar,
        Self::Real,
        Self::CientoVeinte,
        Self::Ochenta,
        Self::Cincuenta,
    ];

    fn actual(e: &Escala) -> Self {
        match e {
            Escala::Ajustar => Self::Ajustar,
            Escala::Real => Self::Real,
            Escala::Porcentaje(p) => match *p as u32 {
                120 => Self::CientoVeinte,
                80 => Self::Ochenta,
                50 => Self::Cincuenta,
                _ => Self::Ajustar,
            },
        }
    }

    fn a_escala(self) -> Escala {
        match self {
            Self::Ajustar => Escala::Ajustar,
            Self::Real => Escala::Real,
            Self::CientoVeinte => Escala::Porcentaje(120.0),
            Self::Ochenta => Escala::Porcentaje(80.0),
            Self::Cincuenta => Escala::Porcentaje(50.0),
        }
    }
}

impl std::fmt::Display for Papel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Papel::A4 => "A4",
            Papel::Letter => "Letter",
            Papel::Foto10x15 => "Foto 4×6",
        })
    }
}

impl std::fmt::Display for Orientacion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Orientacion::Vertical => "Vertical",
            Orientacion::Horizontal => "Horizontal",
        })
    }
}

impl std::fmt::Display for ColorImp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ColorImp::Color => "Color",
            ColorImp::BlancoNegro => "Blanco y negro",
        })
    }
}

impl std::fmt::Display for Calidad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Calidad::Borrador => "Borrador",
            Calidad::Normal => "Normal",
            Calidad::Alta => "Alta",
        })
    }
}

impl std::fmt::Display for Margenes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Margenes::Estrechos => "Estrechos",
            Margenes::Normales => "Normales",
            Margenes::Amplios => "Amplios",
        })
    }
}

impl std::fmt::Display for EscalaPicked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            EscalaPicked::Ajustar => "Ajustar",
            EscalaPicked::Real => "Tamaño real",
            EscalaPicked::CientoVeinte => "120 %",
            EscalaPicked::Ochenta => "80 %",
            EscalaPicked::Cincuenta => "50 %",
        })
    }
}

/// Abrir un documento (diálogo de archivos en un hilo). Las imágenes pasan
/// por la tubería de PDF para imprimir igual que un PDF.
pub fn abrir_documento() -> Result<Documento, String> {
    let elegido: Option<PathBuf> = rfd::FileDialog::new()
        .add_filter("Documentos", &["pdf", "png", "jpg", "jpeg"])
        .set_title("Abrir documento para imprimir")
        .pick_file();
    let Some(ruta) = elegido else {
        return Err("cancelado".into());
    };
    cargar_ruta(&ruta)
}

pub fn cargar_ruta(ruta: &std::path::Path) -> Result<Documento, String> {
    let nombre = ruta
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "documento".into());
    let bytes = std::fs::read(&ruta).map_err(|e| format!("no se pudo leer: {e}"))?;
    let bytes = if bytes.starts_with(b"%PDF-") {
        bytes
    } else {
        pdfsal::imagen_a_pdf(&bytes).map_err(|e| format!("imagen no válida: {e:?}"))?
    };
    let session = exprim::RenderSession::open(bytes.clone().into(), None)
        .map_err(|e| format!("Documento no legible o protegido: {e}"))?;
    let paginas = session.page_count() as u32;
    if paginas == 0 {
        return Err("El documento no tiene páginas".into());
    }
    Ok(Documento {
        nombre,
        bytes,
        paginas,
        ruta: ruta.to_path_buf(),
    })
}

pub fn update(estado: &mut EstadoImprimir, mensaje: MensajeImprimir, dir_datos: &std::path::Path) {
    if estado.imprimiendo && !matches!(mensaje, MensajeImprimir::Impreso(_)) {
        return;
    }
    let necesita_preview = match mensaje {
        MensajeImprimir::Abrir => {
            // El diálogo lo lanza la suscripción global (ui::update lo intercepta).
            false
        }
        MensajeImprimir::DocumentoCargado(Ok(doc)) => {
            estado.abriendo = false;
            estado.documento = Some(doc);
            estado.hoja_activa = 0;
            estado.texto_rango = "Todas".into();
            estado.error_rango = None;
            estado.opciones.rango = impresion::Rango::Todas;
            true
        }
        MensajeImprimir::DocumentoCargado(Err(e)) => {
            estado.abriendo = false;
            if e != "cancelado" {
                estado.aviso = Some(e);
            }
            false
        }
        MensajeImprimir::CambiarPapel(p) => {
            estado.opciones.papel = p;
            true
        }
        MensajeImprimir::CambiarOrientacion(o) => {
            estado.opciones.orientacion = o;
            true
        }
        MensajeImprimir::CambiarColor(c) => {
            estado.opciones.color = c;
            true
        }
        MensajeImprimir::CambiarCalidad(c) => {
            estado.opciones.calidad = c;
            false // la calidad no cambia la vista previa
        }
        MensajeImprimir::CambiarMargenes(m) => {
            estado.opciones.margenes = m;
            true
        }
        MensajeImprimir::CambiarEscala(e) => {
            estado.opciones.escala = e.a_escala();
            true
        }
        MensajeImprimir::PorcentajeEditado(texto) => {
            estado.porcentaje = texto.clone();
            if let Ok(p) = texto.parse::<f32>() {
                if p.is_finite() && (1.0..=400.0).contains(&p) {
                    estado.opciones.escala = Escala::Porcentaje(p);
                }
            }
            true
        }
        MensajeImprimir::CambiarCopias(d) => {
            estado.opciones.copias = match d {
                DeltaCopias::Mas => (estado.opciones.copias + 1).min(99),
                DeltaCopias::Menos => estado.opciones.copias.saturating_sub(1).max(1),
            };
            false
        }
        MensajeImprimir::RangoEditado(t) => {
            estado.texto_rango = t;
            let total = estado.documento.as_ref().map(|d| d.paginas).unwrap_or(0);
            estado.error_rango = impresion::parsear_rango(&estado.texto_rango, total)
                .err()
                .map(|_| "Rango no válido".into());
            false
        }
        MensajeImprimir::RangoEnviado => {
            let total = estado.documento.as_ref().map(|d| d.paginas).unwrap_or(0);
            match impresion::parsear_rango(&estado.texto_rango, total) {
                Ok(r) => {
                    estado.opciones.rango = r;
                    estado.error_rango = None;
                    true
                }
                Err(e) => {
                    estado.error_rango = Some(format!("Rango no válido ({e:?})"));
                    false
                }
            }
        }
        MensajeImprimir::HojasPrevisualizadas(id, _) if id != estado.id_preview => return,
        MensajeImprimir::HojasPrevisualizadas(_, Ok(hojas)) => {
            estado.hojas = hojas.into_iter().map(image::Handle::from_bytes).collect();
            estado.hoja_activa = estado.hoja_activa.min(estado.hojas.len().saturating_sub(1));
            estado.previsualizando = false;
            false
        }
        MensajeImprimir::HojasPrevisualizadas(_, Err(e)) => {
            estado.hojas.clear();
            estado.previsualizando = false;
            estado.aviso = Some(format!("No se pudo abrir el documento: {e}"));
            false
        }
        MensajeImprimir::HojaActiva(i) => {
            if i < estado.hojas.len() {
                estado.hoja_activa = i;
            }
            false
        }
        MensajeImprimir::Zoom(d) => {
            estado.zoom = match d {
                DeltaZoom::Mas => (estado.zoom + 0.15).min(2.5),
                DeltaZoom::Menos => (estado.zoom - 0.15).max(0.25),
                DeltaZoom::Ajustar => 0.75,
            };
            false
        }
        MensajeImprimir::AlternarMiniaturas => {
            estado.ver_miniaturas = !estado.ver_miniaturas;
            false
        }
        MensajeImprimir::AlternarOpciones => {
            estado.ver_opciones = !estado.ver_opciones;
            false
        }
        MensajeImprimir::Imprimir => {
            if estado.documento.is_none()
                || estado.previsualizando
                || estado.error_rango.is_some()
                || estado.hojas.is_empty()
            {
                return;
            }
            let total = estado.documento.as_ref().unwrap().paginas;
            match impresion::parsear_rango(&estado.texto_rango, total) {
                Ok(rango) if rango != estado.opciones.rango => {
                    estado.opciones.rango = rango;
                    estado.previsualizando = true;
                    estado.id_preview += 1;
                    return;
                }
                Err(_) => {
                    estado.error_rango = Some("Rango no válido".into());
                    return;
                }
                _ => {}
            }
            estado.imprimiendo = true;
            estado.id_impresion += 1;
            estado.aviso = None;
            false
        }
        MensajeImprimir::Impreso(res) => {
            estado.imprimiendo = false;
            match res {
                Ok(id) => {
                    if let Some(doc) = &estado.documento {
                        let _ = crate::nucleo::trabajos::registrar(
                            dir_datos,
                            &Trabajo {
                                nombre: doc.ruta.display().to_string(),
                                detalle: format!(
                                    "{} hoja(s) · {}",
                                    estado.hojas.len(),
                                    estado.opciones.papel
                                ),
                                tipo: TipoTrabajo::Impresion,
                                fecha_unix: chrono_offset_now(),
                            },
                        );
                    }
                    estado.aviso = Some(format!("Enviado a imprimir (trabajo {id})"));
                }
                Err(e) => estado.aviso = Some(format!("Error al imprimir: {e}")),
            }
            false
        }
    };
    if necesita_preview && estado.documento.is_some() {
        estado.previsualizando = true;
        estado.id_preview += 1;
    }
}

fn chrono_offset_now() -> i64 {
    chrono::Local::now().timestamp()
}

pub fn vista<'a>(
    t: &Tokens,
    estado: &'a EstadoImprimir,
    impresora: Option<&str>,
    driver_epson: bool,
) -> Element<'a, Mensaje> {
    let (texto_color, secundario, lienzo, borde, superficie) = (
        color(t.texto),
        color(t.texto_secundario),
        color(t.lienzo),
        color(t.borde),
        color(t.superficie),
    );
    let secundario2 = secundario;
    let m = |mi| Mensaje::Imprimir(mi);

    let mut paneles = row![].spacing(10.0);

    if estado.ver_miniaturas {
        let mut minis = column![].spacing(6.0);
        for (i, hoja) in estado.hojas.iter().enumerate() {
            let activa = i == estado.hoja_activa;
            let borde_mini = if activa { color(t.primario) } else { borde };
            minis = minis.push(
                button(
                    container(image(hoja.clone()))
                        .padding(3.0)
                        .style(move |_t| container::Style {
                            background: None,
                            border: iced::border::rounded(6.0).color(borde_mini),
                            ..container::Style::default()
                        }),
                )
                .on_press(m(MensajeImprimir::HojaActiva(i)))
                .style(move |_t, _e| button::Style {
                    background: None,
                    ..button::Style::default()
                }),
            );
        }
        paneles = paneles.push(
            container(
                column![
                    scrollable(minis).height(Length::Fill),
                    button(text("«").size(12.0).color(secundario2))
                        .on_press(m(MensajeImprimir::AlternarMiniaturas))
                        .style(move |_t, _e| button::Style {
                            background: None,
                            text_color: secundario2,
                            ..button::Style::default()
                        }),
                ]
                .spacing(8.0),
            )
            .width(Length::Fixed(96.0))
            .height(Length::Fill)
            .padding(8.0)
            .style(move |_t| container::Style {
                background: Some(superficie.into()),
                border: iced::border::rounded(12.0).color(borde),
                ..container::Style::default()
            }),
        );
    } else {
        paneles = paneles.push(
            button(text("»").size(12.0).color(secundario))
                .on_press(m(MensajeImprimir::AlternarMiniaturas))
                .style(move |_t, _e| button::Style {
                    background: None,
                    text_color: secundario,
                    ..button::Style::default()
                }),
        );
    }

    // Visor central.
    let visor = if estado.documento.is_none() {
        column![
            text("Abre un documento para empezar")
                .size(14.0)
                .color(texto_color),
            text("PDF, PNG o JPG").size(11.0).color(secundario),
            boton_primario(t, "Abrir documento", m(MensajeImprimir::Abrir)),
        ]
        .spacing(10.0)
        .align_x(iced::Alignment::Center)
    } else if estado.previsualizando {
        column![
            text("Calculando vista previa…")
                .size(13.0)
                .color(secundario)
        ]
        .spacing(8.0)
        .align_x(iced::Alignment::Center)
    } else if let Some(hoja) = estado.hojas.get(estado.hoja_activa) {
        let ancho = 460.0 * estado.zoom;
        column![
            container(image(hoja.clone()))
                .width(Length::Fixed(ancho))
                .style(move |_t| container::Style {
                    background: None,
                    border: iced::border::rounded(4.0),
                    ..container::Style::default()
                }),
            row![
                boton_zoom(t, "−", m(MensajeImprimir::Zoom(DeltaZoom::Menos))),
                text(format!("{:.0} %", estado.zoom * 100.0 / 0.75))
                    .size(11.0)
                    .color(secundario),
                boton_zoom(t, "+", m(MensajeImprimir::Zoom(DeltaZoom::Mas))),
                boton_zoom(t, "Ajustar", m(MensajeImprimir::Zoom(DeltaZoom::Ajustar))),
                text(format!(
                    "Hoja {} de {}",
                    estado.hoja_activa + 1,
                    estado.hojas.len()
                ))
                .size(11.0)
                .color(secundario),
            ]
            .spacing(8.0)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(10.0)
        .align_x(iced::Alignment::Center)
    } else {
        column![text("Sin hojas que mostrar").size(13.0).color(secundario)]
    };

    let lienzo2 = lienzo;
    paneles = paneles.push(
        container(scrollable(container(visor).center_x(Length::Fill)).width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_t| container::Style {
                background: Some(lienzo2.into()),
                border: iced::border::rounded(12.0).color(borde),
                ..container::Style::default()
            }),
    );

    // Panel de opciones.
    if estado.ver_opciones {
        let secundario_op = secundario;
        let opciones = column![
            text(
                impresora
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "Sin impresora".into())
            )
            .size(12.0)
            .color(texto_color),
            fila_opcion(
                "Copias",
                secundario_op,
                row![
                    boton_zoom(
                        t,
                        "−",
                        m(MensajeImprimir::CambiarCopias(DeltaCopias::Menos))
                    ),
                    text(estado.opciones.copias.to_string())
                        .size(12.0)
                        .color(texto_color),
                    boton_zoom(t, "+", m(MensajeImprimir::CambiarCopias(DeltaCopias::Mas))),
                ]
                .spacing(6.0),
            ),
            fila_opcion(
                "Papel",
                secundario_op,
                pick_list(
                    vec![Papel::A4, Papel::Letter, Papel::Foto10x15],
                    Some(estado.opciones.papel),
                    move |p| Mensaje::Imprimir(MensajeImprimir::CambiarPapel(p))
                )
                .text_size(11),
            ),
            fila_opcion(
                "Orientación",
                secundario_op,
                pick_list(
                    vec![Orientacion::Vertical, Orientacion::Horizontal],
                    Some(estado.opciones.orientacion),
                    move |o| Mensaje::Imprimir(MensajeImprimir::CambiarOrientacion(o))
                )
                .text_size(11),
            ),
            fila_opcion(
                "Color",
                secundario_op,
                pick_list(
                    vec![ColorImp::Color, ColorImp::BlancoNegro],
                    Some(estado.opciones.color),
                    move |c| Mensaje::Imprimir(MensajeImprimir::CambiarColor(c))
                )
                .text_size(11),
            ),
            fila_opcion(
                "Calidad",
                secundario_op,
                pick_list(
                    if driver_epson {
                        vec![Calidad::Normal, Calidad::Alta]
                    } else {
                        vec![Calidad::Borrador, Calidad::Normal, Calidad::Alta]
                    },
                    Some(estado.opciones.calidad),
                    move |c| Mensaje::Imprimir(MensajeImprimir::CambiarCalidad(c))
                )
                .text_size(11),
            ),
            fila_opcion(
                "Márgenes",
                secundario_op,
                pick_list(
                    vec![Margenes::Estrechos, Margenes::Normales, Margenes::Amplios],
                    Some(estado.opciones.margenes),
                    move |mg| Mensaje::Imprimir(MensajeImprimir::CambiarMargenes(mg))
                )
                .text_size(11),
            ),
            fila_opcion(
                "Escala",
                secundario_op,
                pick_list(
                    EscalaPicked::TODAS.to_vec(),
                    Some(EscalaPicked::actual(&estado.opciones.escala)),
                    move |e| Mensaje::Imprimir(MensajeImprimir::CambiarEscala(e))
                )
                .text_size(11),
            ),
            text("Rango de páginas · pulsa Enter para aplicar").size(10),
            text_input("p. ej. 2-5,8 o Todas", &estado.texto_rango)
                .size(11)
                .on_input(move |t| Mensaje::Imprimir(MensajeImprimir::RangoEditado(t)))
                .on_submit(Mensaje::Imprimir(MensajeImprimir::RangoEnviado)),
            text("Escala personalizada (%)").size(10),
            text_input("Escala % (1–400)", &estado.porcentaje)
                .size(11)
                .on_input(|s| Mensaje::Imprimir(MensajeImprimir::PorcentajeEditado(s))),
            if estado.documento.is_some()
                && !estado.imprimiendo
                && !estado.previsualizando
                && estado.error_rango.is_none()
                && !estado.hojas.is_empty()
                && impresora.is_some()
            {
                boton_primario(t, "Imprimir", m(MensajeImprimir::Imprimir))
            } else if estado.imprimiendo {
                button(text("Enviando…").size(12))
            } else {
                boton_secundario(t, "Abrir documento", m(MensajeImprimir::Abrir))
            },
        ]
        .spacing(4.0);

        let superficie_op = superficie;
        let borde2 = borde;
        paneles = paneles.push(
            container(
                column![
                    scrollable(opciones).height(Length::Fill),
                    button(text("«").size(12.0).color(secundario))
                        .on_press(m(MensajeImprimir::AlternarOpciones))
                        .style(move |_t, _e| button::Style {
                            background: None,
                            text_color: secundario,
                            ..button::Style::default()
                        }),
                ]
                .spacing(8.0),
            )
            .width(Length::Fixed(200.0))
            .height(Length::Fill)
            .padding(10.0)
            .style(move |_t| container::Style {
                background: Some(superficie_op.into()),
                border: iced::border::rounded(12.0).color(borde2),
                ..container::Style::default()
            }),
        );
    } else {
        let s = secundario;
        paneles = paneles.push(
            button(text("«").size(12.0).color(s))
                .on_press(m(MensajeImprimir::AlternarOpciones))
                .style(move |_t, _e| button::Style {
                    background: None,
                    text_color: s,
                    ..button::Style::default()
                }),
        );
    }

    let aviso = estado
        .aviso
        .clone()
        .or_else(|| estado.error_rango.clone())
        .map(|a| text(a).size(11.0).color(color(0xD98A2B)))
        .unwrap_or_else(|| text("").size(11.0));

    let nombre = estado
        .documento
        .as_ref()
        .map(|d| d.nombre.clone())
        .unwrap_or_else(|| "Imprimir".into());
    column![
        row![
            text(nombre).size(18),
            boton_secundario(t, "Abrir", m(MensajeImprimir::Abrir))
        ]
        .spacing(12),
        paneles,
        aviso
    ]
    .spacing(10.0)
    .into()
}

fn fila_opcion<'a, C: Into<Element<'a, Mensaje>>>(
    etiqueta: &str,
    secundario: iced::Color,
    control: C,
) -> iced::widget::Row<'a, Mensaje> {
    row![
        text(etiqueta.to_string()).size(11.0).color(secundario),
        control.into()
    ]
    .spacing(8.0)
    .align_y(iced::Alignment::Center)
    .padding(3.0)
}

fn boton_zoom<'a>(t: &Tokens, etiqueta: &str, al_pulsar: Mensaje) -> button::Button<'a, Mensaje> {
    let secundario = color(t.texto_secundario);
    let borde = color(t.borde);
    button(text(etiqueta.to_string()).size(11.0).color(secundario))
        .padding([3.0, 8.0])
        .style(move |_t, _e| button::Style {
            background: None,
            border: iced::border::rounded(7.0).color(borde),
            text_color: secundario,
            ..button::Style::default()
        })
        .on_press(al_pulsar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nucleo::impresion::Rango;
    fn documento() -> Documento {
        let mut pdf = printpdf::PdfDocument::new("prueba");
        pdf.pages = (0..3)
            .map(|_| printpdf::PdfPage::new(printpdf::Mm(210.0), printpdf::Mm(297.0), vec![]))
            .collect();
        Documento {
            nombre: "tres.pdf".into(),
            bytes: pdf.save(&Default::default(), &mut vec![]),
            paginas: 3,
            ruta: PathBuf::from("tres.pdf"),
        }
    }
    #[test]
    fn rango_se_valida_con_documento_no_con_preview_filtrada() {
        let dir = tempfile::tempdir().unwrap();
        let mut e = EstadoImprimir::default();
        update(
            &mut e,
            MensajeImprimir::DocumentoCargado(Ok(documento())),
            dir.path(),
        );
        e.hojas = vec![
            image::Handle::from_bytes(vec![1]),
            image::Handle::from_bytes(vec![1]),
        ];
        update(
            &mut e,
            MensajeImprimir::RangoEditado("3".into()),
            dir.path(),
        );
        update(&mut e, MensajeImprimir::RangoEnviado, dir.path());
        assert!(e.error_rango.is_none());
        assert_eq!(e.opciones.rango, Rango::Paginas(vec![(3, 3)]));
    }
    #[test]
    fn rango_invalido_no_se_imprime() {
        let dir = tempfile::tempdir().unwrap();
        let mut e = EstadoImprimir::default();
        update(
            &mut e,
            MensajeImprimir::DocumentoCargado(Ok(documento())),
            dir.path(),
        );
        e.previsualizando = false;
        update(
            &mut e,
            MensajeImprimir::RangoEditado("99".into()),
            dir.path(),
        );
        update(&mut e, MensajeImprimir::Imprimir, dir.path());
        assert!(!e.imprimiendo);
        assert!(e.error_rango.is_some());
    }
}

//! Estado global, mensajes, navegación y composición de la ventana.

pub mod hilos;
pub mod herramientas;
pub mod escanear;
pub mod imprimir;
pub mod inicio;
pub mod tema;
pub mod widgets;

use std::path::PathBuf;

use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length, Subscription};

use crate::nucleo::dispositivos::{self, Dispositivos};
use crate::nucleo::impresion as nucleo_impresion;
use crate::nucleo::trabajos::{self, Trabajo};

use tema::Tokens;
use widgets::color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pagina {
    Inicio,
    Imprimir,
    Escanear,
    Herramientas,
}

impl Pagina {
    pub fn titulo(self) -> &'static str {
        match self {
            Pagina::Inicio => "Inicio",
            Pagina::Imprimir => "Imprimir",
            Pagina::Escanear => "Escanear",
            Pagina::Herramientas => "Herramientas",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mensaje {
    Ir(Pagina),
    AlternarTema,
    DispositivosDetectados(Dispositivos),
    Imprimir(imprimir::MensajeImprimir),
    Escanear(escanear::MensajeEscanear),
    Herramientas(herramientas::MensajeHerramientas),
}

#[derive(Debug)]
pub struct Estado {
    pub modo_oscuro: bool,
    pub pagina: Pagina,
    pub dispositivos: Option<Dispositivos>,
    pub recientes: Vec<Trabajo>,
    pub dir_datos: PathBuf,
    pub imp: imprimir::EstadoImprimir,
    pub scan: escanear::EstadoEscanear,
    pub dni: herramientas::EstadoDni,
}

pub fn dir_datos() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("printdesk")
}

pub fn boot() -> Estado {
    let dir_datos = dir_datos();
    let recientes = trabajos::recientes(&dir_datos, 20);
    Estado {
        modo_oscuro: false,
        pagina: Pagina::Inicio,
        dispositivos: None,
        recientes,
        dir_datos,
        imp: imprimir::EstadoImprimir::default(),
        scan: escanear::EstadoEscanear::default(),
        dni: herramientas::EstadoDni::default(),
    }
}

pub fn update(estado: &mut Estado, mensaje: Mensaje) {
    match mensaje {
        Mensaje::Ir(p) => estado.pagina = p,
        Mensaje::AlternarTema => estado.modo_oscuro = !estado.modo_oscuro,
        Mensaje::DispositivosDetectados(d) => estado.dispositivos = Some(d),
        // Abrir lanza el diálogo en un hilo (suscripción); el resto es suyo.
        Mensaje::Imprimir(imprimir::MensajeImprimir::Abrir) => {
            estado.imp.abriendo = true;
            estado.imp.id_abrir += 1;
        }
        Mensaje::Imprimir(m) => {
            let dir = estado.dir_datos.clone();
            imprimir::update(&mut estado.imp, m, &dir);
        }
        Mensaje::Escanear(escanear::MensajeEscanear::Escanear) if estado.dispositivos.as_ref().is_none_or(|d| d.escaners.is_empty()) => {
            estado.scan.aviso = Some("No se detecta un escáner. Conecta la Epson y actualiza los dispositivos.".into());
        }
        Mensaje::Escanear(escanear::MensajeEscanear::Escanear) if estado.dni.escaneando() => estado.scan.aviso = Some("Termina la captura del DNI antes de escanear otra hoja.".into()),
        Mensaje::Escanear(m) => {
            if let escanear::MensajeEscanear::Guardado(Ok(ref ruta)) = m {
                let _ = trabajos::registrar(&estado.dir_datos, &Trabajo { nombre: ruta.display().to_string(), detalle: format!("{} páginas", estado.scan.paginas.len()), tipo: trabajos::TipoTrabajo::Escaneo, fecha_unix: chrono::Local::now().timestamp() });
                estado.recientes = trabajos::recientes(&estado.dir_datos, 20);
            }
            escanear::update(&mut estado.scan, m);
        }
        Mensaje::Herramientas(herramientas::MensajeHerramientas::Escanear)
            if estado.scan.escaneando || estado.dispositivos.as_ref().is_none_or(|d| d.escaners.is_empty()) => {
                estado.dni.aviso = "Escáner no disponible o ya ocupado. Conecta la Epson y actualiza dispositivos.".into();
            }
        Mensaje::Herramientas(m) => {
            if let herramientas::MensajeHerramientas::Guardado(Ok(ref ruta)) = m {
                let _ = trabajos::registrar(&estado.dir_datos, &Trabajo { nombre: ruta.display().to_string(), detalle: "DNI · 2 páginas A4".into(), tipo: trabajos::TipoTrabajo::Escaneo, fecha_unix: chrono::Local::now().timestamp() });
                estado.recientes = trabajos::recientes(&estado.dir_datos, 20);
            }
            herramientas::update(&mut estado.dni, m);
        }
    }
}

fn impresora_actual(estado: &Estado) -> Option<String> {
    estado
        .dispositivos
        .as_ref()?
        .impresoras
        .iter()
        .find(|i| i.por_defecto)
        .or_else(|| estado.dispositivos.as_ref()?.impresoras.first())
        .map(|i| i.nombre.clone())
}

pub fn subscription(estado: &Estado) -> Subscription<Mensaje> {
    use imprimir::MensajeImprimir as MI;
    use iced::Subscription;

    let mut lotes: Vec<Subscription<Mensaje>> = Vec::new();

    if estado.dispositivos.is_none() {
        lotes.push(hilos::trabajo(1, dispositivos::detectar).map(envolver_dispositivos));
    }

    if let Some(doc) = &estado.imp.documento {
        // Vista previa: relanzada con id nuevo al cambiar opciones o documento.
        if estado.imp.previsualizando {
            let bytes = doc.bytes.clone();
            let opts = estado.imp.opciones.clone();
            lotes.push(
                hilos::trabajo(1000 + estado.imp.id_preview, move || {
                    nucleo_impresion::previsualizar(&bytes, &opts).map_err(|e| e.to_string())
                })
                .map(|res| Mensaje::Imprimir(MI::HojasPrevisualizadas(res))),
            );
        }
        // Envío real a CUPS.
        if estado.imp.imprimiendo {
            if let Some(impresora) = impresora_actual(estado) {
                let bytes = doc.bytes.clone();
                let titulo = doc.nombre.clone();
                let opts = estado.imp.opciones.clone();
                lotes.push(
                    hilos::trabajo(2000 + estado.imp.id_impresion, move || {
                        let tmp = std::env::temp_dir().join("printdesk");
                        let _ = std::fs::create_dir_all(&tmp);
                        nucleo_impresion::imprimir_cups(
                            &impresora,
                            &bytes,
                            &titulo,
                            &opts,
                            "lp",
                            &tmp,
                        )
                        .map_err(|e| e.to_string())
                    })
                    .map(|res| Mensaje::Imprimir(MI::Impreso(res))),
                );
            }
        }
    }

    if estado.imp.abriendo {
        lotes.push(
            hilos::trabajo(3000 + estado.imp.id_abrir, imprimir::abrir_documento)
                .map(|res| Mensaje::Imprimir(MI::DocumentoCargado(res))),
        );
    }

    if estado.scan.escaneando {
        if let Some(dispositivo) = estado.dispositivos.as_ref().and_then(|d| d.escaners.first()) {
            let id = dispositivo.id.clone();
            let opts = estado.scan.opciones;
            let cancel = estado.scan.cancel.clone();
            lotes.push(hilos::trabajo(4000 + estado.scan.id, move || escanear::capturar(id, opts, cancel))
                .map(|res| Mensaje::Escanear(escanear::MensajeEscanear::Lista(res))));
        }
    }
    if estado.scan.guardando {
        let paginas: Vec<_> = estado.scan.paginas.iter().map(|p| p.pagina.clone()).collect();
        let opts = estado.scan.opciones;
        lotes.push(hilos::trabajo(5000 + estado.scan.id, move || escanear::guardar(&paginas, opts.formato_salida, opts.resolucion as u16, &escanear::carpeta(), &chrono::Local::now().format("%Y-%m-%d_%H%M%S").to_string()))
            .map(|res| Mensaje::Escanear(escanear::MensajeEscanear::Guardado(res))));
    }
    if estado.dni.escaneando() {
        if let Some(d) = estado.dispositivos.as_ref().and_then(|d| d.escaners.first()) {
            let id = d.id.clone(); let cancel = estado.dni.cancel.clone();
            lotes.push(hilos::trabajo(6000 + estado.dni.id, move || escanear::capturar(id, crate::nucleo::escaneo::OpcionesEscaneo::predeterminadas(), cancel))
                .map(|res| Mensaje::Herramientas(herramientas::MensajeHerramientas::Lista(res))));
        }
    }
    if estado.dni.guardando {
        let paginas: Vec<_> = [estado.dni.asistente.anverso(), estado.dni.asistente.reverso()].into_iter().flatten().cloned().collect();
        lotes.push(hilos::trabajo(7000 + estado.dni.id, move || escanear::guardar(&paginas, crate::nucleo::escaneo::FormatoSalida::Pdf, 300, &escanear::carpeta(), &chrono::Local::now().format("DNI_%Y-%m-%d_%H%M%S").to_string()))
            .map(|res| Mensaje::Herramientas(herramientas::MensajeHerramientas::Guardado(res))));
    }

    Subscription::batch(lotes)
}

fn envolver_dispositivos(d: Dispositivos) -> Mensaje {
    Mensaje::DispositivosDetectados(d)
}

pub fn view(estado: &Estado) -> Element<'_, Mensaje> {
    let t = if estado.modo_oscuro {
        Tokens::oscuro()
    } else {
        Tokens::claro()
    };

    let lateral = barra_lateral(&t, estado.pagina);

    let contenido = container(match estado.pagina {
        Pagina::Inicio => inicio::vista(&t, &estado.dispositivos, &estado.recientes),
        Pagina::Imprimir => {
            imprimir::vista(&t, &estado.imp, impresora_actual(estado).as_deref())
        }
        Pagina::Escanear => escanear::vista(&t, &estado.scan),
        Pagina::Herramientas => herramientas::vista(&t, &estado.dni),
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(20.0)
    .style(move |_t| container::Style {
        background: Some(color(t.fondo).into()),
        ..container::Style::default()
    });

    row![lateral, contenido].into()
}

fn barra_lateral<'a>(t: &Tokens, actual: Pagina) -> Element<'a, Mensaje> {
    let nav = |etiqueta: &str, pagina: Pagina| {
        let activa = pagina == actual;
        let fondo = if activa { color(0x3A46D6) } else { Color::TRANSPARENT };
        let letra = if activa { color(0xCDD3FF) } else { color(0xA6ACC2) };
        button(
            container(text(etiqueta.to_string()).size(12.0).color(letra))
                .center_x(Length::Fill)
                .padding(6.0),
        )
        .width(Length::Fill)
        .style(move |_t, _e| button::Style {
            background: Some(fondo.into()),
            border: iced::border::rounded(8.0),
            text_color: letra,
            ..button::Style::default()
        })
        .on_press(Mensaje::Ir(pagina))
    };

    let lado = color(t.lado);
    container(
        column![
            text("PrintDesk").size(14.0).color(color(0xF0F2FA)),
            nav("Inicio", Pagina::Inicio),
            nav("Imprimir", Pagina::Imprimir),
            nav("Escanear", Pagina::Escanear),
            nav("Herramientas", Pagina::Herramientas),
            button(
                text(if *t == Tokens::oscuro() {
                    "Modo claro"
                } else {
                    "Modo oscuro"
                }
                .to_string())
                .size(11.0)
                .color(color(0xA6ACC2))
            )
            .style(|_t, _e| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                text_color: color(0xA6ACC2),
                ..button::Style::default()
            })
            .on_press(Mensaje::AlternarTema),
        ]
        .spacing(5.0)
        .padding(12.0),
    )
    .width(Length::Fixed(148.0))
    .height(Length::Fill)
    .style(move |_t| container::Style {
        background: Some(lado.into()),
        ..container::Style::default()
    })
    .into()
}

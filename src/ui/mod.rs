//! Estado global, mensajes, navegación y composición de la ventana.

pub mod escanear;
pub mod herramientas;
pub mod hilos;
pub mod iconos;
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
    TemaSistema(iced::theme::Mode),
    DispositivosDetectados(Dispositivos),
    RefrescarDispositivos,
    Reabrir(PathBuf),
    SolicitarCerrar,
    ConfirmarCerrar,
    CancelarCerrar,
    Imprimir(imprimir::MensajeImprimir),
    Escanear(escanear::MensajeEscanear),
    Herramientas(herramientas::MensajeHerramientas),
}

#[derive(Debug)]
pub struct Estado {
    pub modo_oscuro: bool,
    pub seguir_tema: bool,
    pub pagina: Pagina,
    pub dispositivos: Option<Dispositivos>,
    pub recientes: Vec<Trabajo>,
    pub dir_datos: PathBuf,
    pub imp: imprimir::EstadoImprimir,
    pub scan: escanear::EstadoEscanear,
    pub dni: herramientas::EstadoDni,
    pub id_dispositivos: u64,
    pub confirmar_cierre: bool,
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
        seguir_tema: true,
        pagina: Pagina::Inicio,
        dispositivos: None,
        recientes,
        dir_datos,
        imp: imprimir::EstadoImprimir::default(),
        scan: escanear::EstadoEscanear::default(),
        dni: herramientas::EstadoDni::default(),
        id_dispositivos: 1,
        confirmar_cierre: false,
    }
}

pub fn update(estado: &mut Estado, mensaje: Mensaje) -> iced::Task<Mensaje> {
    match mensaje {
        Mensaje::ConfirmarCerrar => {
            estado
                .scan
                .cancel
                .store(true, std::sync::atomic::Ordering::SeqCst);
            estado
                .dni
                .cancel
                .store(true, std::sync::atomic::Ordering::SeqCst);
            return iced::exit();
        }
        Mensaje::CancelarCerrar => estado.confirmar_cierre = false,
        Mensaje::SolicitarCerrar => {
            if estado.scan.paginas.is_empty()
                && !estado.scan.escaneando
                && !estado.scan.guardando
                && estado.dni.asistente.anverso().is_none()
                && !estado.dni.escaneando()
                && !estado.imp.imprimiendo
            {
                return iced::exit();
            }
            estado.confirmar_cierre = true;
        }
        Mensaje::Ir(p) => estado.pagina = p,
        Mensaje::AlternarTema => {
            estado.modo_oscuro = !estado.modo_oscuro;
            estado.seguir_tema = false;
        }
        Mensaje::TemaSistema(modo) if estado.seguir_tema => {
            estado.modo_oscuro = modo == iced::theme::Mode::Dark
        }
        Mensaje::TemaSistema(_) => {}
        Mensaje::DispositivosDetectados(d) => estado.dispositivos = Some(d),
        Mensaje::RefrescarDispositivos
            if !estado.scan.escaneando && !estado.dni.escaneando() && !estado.imp.imprimiendo =>
        {
            estado.dispositivos = None;
            estado.id_dispositivos += 1;
        }
        Mensaje::RefrescarDispositivos => {}
        // Abrir lanza el diálogo en un hilo (suscripción); el resto es suyo.
        Mensaje::Imprimir(imprimir::MensajeImprimir::Abrir)
            if !estado.imp.imprimiendo && !estado.imp.abriendo =>
        {
            estado.imp.ruta_abrir = None;
            estado.imp.abriendo = true;
            estado.imp.id_abrir += 1;
        }
        Mensaje::Reabrir(ruta) if !estado.imp.imprimiendo && !estado.imp.abriendo => {
            estado.pagina = Pagina::Imprimir;
            estado.imp.ruta_abrir = Some(ruta);
            estado.imp.abriendo = true;
            estado.imp.id_abrir += 1;
        }
        Mensaje::Reabrir(_) => {}
        Mensaje::Imprimir(imprimir::MensajeImprimir::Imprimir)
            if impresora_actual(estado).is_none() =>
        {
            estado.imp.aviso = Some("No hay una impresora disponible".into())
        }
        Mensaje::Imprimir(m) => {
            let dir = estado.dir_datos.clone();
            imprimir::update(&mut estado.imp, m, &dir);
            estado.recientes = trabajos::recientes(&estado.dir_datos, 20);
        }
        Mensaje::Escanear(escanear::MensajeEscanear::Escanear)
            if estado
                .dispositivos
                .as_ref()
                .is_none_or(|d| d.escaners.is_empty()) =>
        {
            estado.scan.aviso = Some(
                "No se detecta un escáner. Conecta la Epson y actualiza los dispositivos.".into(),
            );
        }
        Mensaje::Escanear(escanear::MensajeEscanear::Escanear) if estado.dni.escaneando() => {
            estado.scan.aviso =
                Some("Termina la captura del DNI antes de escanear otra hoja.".into())
        }
        Mensaje::Escanear(m) => {
            if let escanear::MensajeEscanear::Guardado(Ok(ref ruta)) = m {
                let _ = trabajos::registrar(
                    &estado.dir_datos,
                    &Trabajo {
                        nombre: ruta.display().to_string(),
                        detalle: format!("{} páginas", estado.scan.paginas.len()),
                        tipo: trabajos::TipoTrabajo::Escaneo,
                        fecha_unix: chrono::Local::now().timestamp(),
                    },
                );
                estado.recientes = trabajos::recientes(&estado.dir_datos, 20);
            }
            escanear::update(&mut estado.scan, m);
        }
        Mensaje::Herramientas(herramientas::MensajeHerramientas::Escanear)
            if estado.scan.escaneando
                || estado
                    .dispositivos
                    .as_ref()
                    .is_none_or(|d| d.escaners.is_empty()) =>
        {
            estado.dni.aviso =
                "Escáner no disponible o ya ocupado. Conecta la Epson y actualiza dispositivos."
                    .into();
        }
        Mensaje::Herramientas(m) => {
            if let herramientas::MensajeHerramientas::Guardado(Ok(ref ruta)) = m {
                let _ = trabajos::registrar(
                    &estado.dir_datos,
                    &Trabajo {
                        nombre: ruta.display().to_string(),
                        detalle: "DNI · 2 páginas A4".into(),
                        tipo: trabajos::TipoTrabajo::Escaneo,
                        fecha_unix: chrono::Local::now().timestamp(),
                    },
                );
                estado.recientes = trabajos::recientes(&estado.dir_datos, 20);
            }
            herramientas::update(&mut estado.dni, m);
        }
    }
    iced::Task::none()
}

fn impresora_actual(estado: &Estado) -> Option<String> {
    estado
        .dispositivos
        .as_ref()?
        .impresoras
        .iter()
        .find(|i| i.nombre.to_lowercase().contains("l375"))
        .or_else(|| {
            estado
                .dispositivos
                .as_ref()?
                .impresoras
                .iter()
                .find(|i| i.por_defecto)
        })
        .or_else(|| estado.dispositivos.as_ref()?.impresoras.first())
        .map(|i| i.nombre.clone())
}

pub fn subscription(estado: &Estado) -> Subscription<Mensaje> {
    use iced::Subscription;
    use imprimir::MensajeImprimir as MI;

    let mut lotes: Vec<Subscription<Mensaje>> = Vec::new();
    lotes.push(iced::system::theme_changes().map(Mensaje::TemaSistema));

    if estado.dispositivos.is_none() {
        lotes.push(
            hilos::trabajo(estado.id_dispositivos, dispositivos::detectar)
                .map(envolver_dispositivos),
        );
    }

    if let Some(doc) = &estado.imp.documento {
        // Vista previa: relanzada con id nuevo al cambiar opciones o documento.
        if estado.imp.previsualizando {
            let bytes = doc.bytes.clone();
            let opts = estado.imp.opciones.clone();
            let id = estado.imp.id_preview;
            lotes.push(
                hilos::trabajo(1000 + estado.imp.id_preview, move || {
                    nucleo_impresion::previsualizar(&bytes, &opts).map_err(|e| e.to_string())
                })
                .map(move |res| Mensaje::Imprimir(MI::HojasPrevisualizadas(id, res))),
            );
        }
        // Envío real a CUPS.
        if estado.imp.imprimiendo {
            if let Some(impresora) = impresora_actual(estado) {
                let driver = estado
                    .dispositivos
                    .as_ref()
                    .and_then(|d| d.impresoras.iter().find(|p| p.nombre == impresora))
                    .map(|p| p.driver.clone())
                    .unwrap_or_default();
                let bytes = doc.bytes.clone();
                let titulo = doc.nombre.clone();
                let opts = estado.imp.opciones.clone();
                lotes.push(
                    hilos::trabajo(2000 + estado.imp.id_impresion, move || {
                        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
                        nucleo_impresion::imprimir_cups(
                            &impresora,
                            &bytes,
                            &titulo,
                            &opts,
                            "lp",
                            tmp.path(),
                            &driver,
                        )
                        .map_err(|e| e.to_string())
                    })
                    .map(|res| Mensaje::Imprimir(MI::Impreso(res))),
                );
            }
        }
    }

    if estado.imp.abriendo {
        let ruta = estado.imp.ruta_abrir.clone();
        lotes.push(
            hilos::trabajo(3000 + estado.imp.id_abrir, move || match ruta {
                Some(p) => imprimir::cargar_ruta(&p),
                None => imprimir::abrir_documento(),
            })
            .map(|res| Mensaje::Imprimir(MI::DocumentoCargado(res))),
        );
    }
    lotes.push(iced::event::listen().filter_map(|evento| match evento {
        iced::Event::Window(iced::window::Event::FileDropped(ruta)) => Some(Mensaje::Reabrir(ruta)),
        iced::Event::Window(iced::window::Event::CloseRequested) => Some(Mensaje::SolicitarCerrar),
        _ => None,
    }));

    if estado.scan.escaneando {
        if let Some(dispositivo) = estado
            .dispositivos
            .as_ref()
            .and_then(|d| d.escaners.first())
        {
            let id = dispositivo.id.clone();
            let opts = estado.scan.opciones;
            let cancel = estado.scan.cancel.clone();
            lotes.push(
                hilos::trabajo(4000 + estado.scan.id, move || {
                    escanear::capturar(id, opts, cancel)
                })
                .map(|res| Mensaje::Escanear(escanear::MensajeEscanear::Lista(res))),
            );
        }
    }
    if estado.scan.guardando {
        let paginas: Vec<_> = estado
            .scan
            .paginas
            .iter()
            .map(|p| p.pagina.clone())
            .collect();
        let opts = estado.scan.opciones;
        lotes.push(
            hilos::trabajo(5000 + estado.scan.id, move || {
                escanear::guardar(
                    &paginas,
                    opts.formato_salida,
                    opts.resolucion as u16,
                    &escanear::carpeta(),
                    &chrono::Local::now().format("%Y-%m-%d_%H%M%S").to_string(),
                )
            })
            .map(|res| Mensaje::Escanear(escanear::MensajeEscanear::Guardado(res))),
        );
    }
    if estado.dni.escaneando() {
        if let Some(d) = estado
            .dispositivos
            .as_ref()
            .and_then(|d| d.escaners.first())
        {
            let id = d.id.clone();
            let cancel = estado.dni.cancel.clone();
            lotes.push(
                hilos::trabajo(6000 + estado.dni.id, move || {
                    escanear::capturar(
                        id,
                        crate::nucleo::escaneo::OpcionesEscaneo::predeterminadas(),
                        cancel,
                    )
                })
                .map(|res| Mensaje::Herramientas(herramientas::MensajeHerramientas::Lista(res))),
            );
        }
    }
    if estado.dni.guardando {
        let paginas: Vec<_> = [
            estado.dni.asistente.anverso(),
            estado.dni.asistente.reverso(),
        ]
        .into_iter()
        .flatten()
        .cloned()
        .collect();
        lotes.push(
            hilos::trabajo(7000 + estado.dni.id, move || {
                escanear::guardar(
                    &paginas,
                    crate::nucleo::escaneo::FormatoSalida::Pdf,
                    300,
                    &escanear::carpeta(),
                    &chrono::Local::now()
                        .format("DNI_%Y-%m-%d_%H%M%S")
                        .to_string(),
                )
            })
            .map(|res| Mensaje::Herramientas(herramientas::MensajeHerramientas::Guardado(res))),
        );
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
            let nombre = impresora_actual(estado);
            let epson = estado.dispositivos.as_ref().is_some_and(|d| {
                d.impresoras.iter().any(|p| {
                    Some(&p.nombre) == nombre.as_ref() && p.driver.contains("PLAIN_NORMAL")
                })
            });
            imprimir::vista(&t, &estado.imp, nombre.as_deref(), epson)
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

    let base = row![lateral, contenido];
    if estado.confirmar_cierre {
        let dialogo = widgets::tarjeta(&t, column![text("¿Cerrar PrintDesk?").size(22), text("Hay páginas en la sesión o una operación en curso. Las páginas no guardadas se perderán."),
            row![widgets::boton_primario(&t, "Seguir trabajando", Mensaje::CancelarCerrar), widgets::boton_secundario(&t, "Cerrar igualmente", Mensaje::ConfirmarCerrar)].spacing(8)].spacing(12));
        iced::widget::stack![
            base,
            container(dialogo)
                .center(Length::Fill)
                .padding(64)
                .style(|_| container::Style {
                    background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                    ..Default::default()
                })
        ]
        .into()
    } else {
        base.into()
    }
}

pub fn theme(e: &Estado) -> iced::Theme {
    let t = if e.modo_oscuro {
        Tokens::oscuro()
    } else {
        Tokens::claro()
    };
    iced::Theme::custom(
        "PrintDesk",
        iced::theme::Palette {
            background: color(t.fondo),
            text: color(t.texto),
            primary: color(t.primario),
            success: color(t.correcto),
            danger: color(0xD14B61),
            warning: color(0xD98A2B),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navegacion_y_cambio_de_tema_por_click() {
        let mut e = boot();
        e.dispositivos = Some(Dispositivos::default());
        e.recientes.clear();
        let mut sim = iced_test::simulator(view(&e));
        sim.click("Escanear").unwrap();
        let mensajes: Vec<_> = sim.into_messages().collect();
        for m in mensajes {
            let _ = update(&mut e, m);
        }
        assert_eq!(e.pagina, Pagina::Escanear);
        let mut sim = iced_test::simulator(view(&e));
        sim.click("Modo oscuro").unwrap();
        for m in sim.into_messages() {
            let _ = update(&mut e, m);
        }
        assert!(e.modo_oscuro);
        assert!(
            iced_test::simulator(view(&e))
                .find("Escanear página")
                .is_ok()
        );
    }

    #[test]
    fn cerrar_con_paginas_pide_confirmacion() {
        let mut e = boot();
        e.scan.escaneando = true;
        let _ = update(&mut e, Mensaje::SolicitarCerrar);
        assert!(e.confirmar_cierre);
    }

    #[test]
    #[ignore = "renderiza capturas headless para revisión visual"]
    fn capturas_pantallas_claras_y_oscuras() {
        for oscuro in [false, true] {
            for pagina in [
                Pagina::Inicio,
                Pagina::Imprimir,
                Pagina::Escanear,
                Pagina::Herramientas,
            ] {
                let mut e = boot();
                e.dispositivos = Some(Dispositivos::default());
                e.recientes.clear();
                e.pagina = pagina;
                e.modo_oscuro = oscuro;
                let settings = iced::Settings {
                    fonts: vec![
                        include_bytes!("../../assets/Inter-Regular.ttf")
                            .as_slice()
                            .into(),
                    ],
                    default_font: iced::Font::with_name("Inter"),
                    ..Default::default()
                };
                let mut sim = iced_test::Simulator::with_size(
                    settings,
                    iced::Size::new(1080.0, 720.0),
                    view(&e),
                );
                let snapshot = sim.snapshot(&theme(&e)).unwrap();
                let path = format!(
                    "/tmp/opencode/printdesk-{:?}-{}",
                    pagina,
                    if oscuro { "oscuro" } else { "claro" }
                );
                assert!(snapshot.matches_image(path).unwrap());
            }
        }
    }
}

fn barra_lateral<'a>(t: &Tokens, actual: Pagina) -> Element<'a, Mensaje> {
    let nav = |etiqueta: &str, pagina: Pagina| {
        let activa = pagina == actual;
        let fondo = if activa {
            color(0x3A46D6)
        } else {
            Color::TRANSPARENT
        };
        let letra = if activa {
            color(0xCDD3FF)
        } else {
            color(0xA6ACC2)
        };
        button(
            container(
                row![
                    iconos::icono(
                        match pagina {
                            Pagina::Inicio => "inicio",
                            Pagina::Escanear => "escanear",
                            Pagina::Herramientas => "herramientas",
                            _ => "imprimir",
                        },
                        letra
                    ),
                    text(etiqueta.to_string()).size(12.0).color(letra)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
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
            row![
                iconos::icono("imprimir", color(0x8993FF)),
                text("PrintDesk").size(17.0).color(color(0xF0F2FA))
            ]
            .spacing(8),
            nav("Inicio", Pagina::Inicio),
            nav("Imprimir", Pagina::Imprimir),
            nav("Escanear", Pagina::Escanear),
            nav("Herramientas", Pagina::Herramientas),
            button(
                text(
                    if *t == Tokens::oscuro() {
                        "Modo claro"
                    } else {
                        "Modo oscuro"
                    }
                    .to_string()
                )
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

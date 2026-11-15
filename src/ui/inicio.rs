//! Pantalla de Inicio: estado real de los dispositivos, accesos rápidos
//! y trabajos recientes.

use chrono::TimeZone as _;
use iced::Element;
use iced::widget::{column, container, row, text};

use crate::nucleo::dispositivos::Dispositivos;
use crate::nucleo::trabajos::{TipoTrabajo, Trabajo};

use super::tema::Tokens;
use super::widgets::{boton_primario, boton_secundario, chip, color, tarjeta};
use crate::ui::{Mensaje, Pagina};

pub fn vista(
    t: &Tokens,
    dispositivos: &Option<Dispositivos>,
    recientes: &[Trabajo],
) -> Element<'static, Mensaje> {
    let (nombre, imprime, escanea) = match dispositivos {
        Some(d) => {
            let imprime = d.impresoras.iter().any(|i| i.acepta);
            let escanea = !d.escaners.is_empty();
            let nombre = d
                .impresoras
                .first()
                .map(|i| i.nombre.replace('_', " "))
                .unwrap_or_else(|| "Sin impresora".into());
            (nombre, imprime, escanea)
        }
        None => ("Detectando dispositivos…".to_string(), false, false),
    };

    let lienzo_icono = color(t.lienzo);
    let tarjeta_dispositivo = tarjeta(
        t,
        row![
            container(super::iconos::icono("imprimir", color(t.primario)))
                .width(64.0)
                .height(52.0)
                .center(iced::Length::Fixed(52.0))
                .style(move |_t| container::Style {
                    background: Some(lienzo_icono.into()),
                    border: iced::border::rounded(10.0),
                    ..container::Style::default()
                }),
            column![
                text(nombre.clone()).size(16.0).color(color(t.texto)),
                row![
                    chip(
                        t,
                        if imprime {
                            "Cola CUPS acepta trabajos"
                        } else {
                            "Cola no disponible"
                        },
                        imprime
                    ),
                    chip(
                        t,
                        if escanea {
                            "Escáner detectado"
                        } else {
                            "Escáner no detectado"
                        },
                        escanea
                    ),
                ]
                .spacing(8.0)
            ]
            .spacing(8.0)
        ]
        .spacing(14.0)
        .align_y(iced::Alignment::Center),
    );

    let accesos = row![
        boton_primario(t, "Imprimir documento", Mensaje::Ir(Pagina::Imprimir)),
        boton_secundario(t, "Escanear", Mensaje::Ir(Pagina::Escanear)),
        boton_secundario(t, "Asistente DNI", Mensaje::Ir(Pagina::Herramientas)),
    ]
    .spacing(8.0);

    let recientes_ui = if recientes.is_empty() {
        tarjeta(
            t,
            column![
                text("Recientes")
                    .size(12.0)
                    .color(color(t.texto_secundario)),
                text("Aún no hay trabajos: lo que imprimas o escanees aparecerá aquí.")
                    .size(12.0)
                    .color(color(t.texto_secundario)),
            ]
            .spacing(6.0),
        )
    } else {
        let mut lista = column![
            text("Recientes")
                .size(12.0)
                .color(color(t.texto_secundario))
        ]
        .spacing(4.0);
        for trabajo in recientes.iter().take(5) {
            let etiqueta = match trabajo.tipo {
                TipoTrabajo::Impresion => "impreso",
                TipoTrabajo::Escaneo => "escaneado",
            };
            let fecha = chrono::Local
                .timestamp_opt(trabajo.fecha_unix, 0)
                .single()
                .map(|f| f.format("%d/%m %H:%M").to_string())
                .unwrap_or_default();
            lista = lista.push(
                row![
                    text(
                        std::path::Path::new(&trabajo.nombre)
                            .file_name()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_else(|| trabajo.nombre.clone())
                    )
                    .size(12.0)
                    .color(color(t.texto)),
                    text(format!("{} · {} · {}", trabajo.detalle, etiqueta, fecha))
                        .size(11.0)
                        .color(color(t.texto_secundario)),
                    boton_secundario(
                        t,
                        "Reabrir",
                        Mensaje::Reabrir(std::path::PathBuf::from(&trabajo.nombre))
                    ),
                ]
                .spacing(8.0)
                .align_y(iced::Alignment::Center),
            );
        }
        tarjeta(t, lista)
    };

    column![tarjeta_dispositivo, accesos, recientes_ui]
        .push(boton_secundario(t, "Actualizar dispositivos", Mensaje::RefrescarDispositivos))
        .push(text("El estado de la cola no garantiza que haya papel o conexión física. No se estiman niveles de tinta.").size(11))
        .spacing(10.0)
        .into()
}

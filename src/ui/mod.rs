//! Estado global, mensajes, navegación y composición de la ventana.

pub mod hilos;
pub mod inicio;
pub mod tema;
pub mod widgets;

use std::path::PathBuf;

use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length, Subscription};

use crate::nucleo::dispositivos::{self, Dispositivos};
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
}

#[derive(Debug)]
pub struct Estado {
    pub modo_oscuro: bool,
    pub pagina: Pagina,
    pub dispositivos: Option<Dispositivos>,
    pub recientes: Vec<Trabajo>,
    pub dir_datos: PathBuf,
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
    }
}

pub fn update(estado: &mut Estado, mensaje: Mensaje) {
    match mensaje {
        Mensaje::Ir(p) => estado.pagina = p,
        Mensaje::AlternarTema => estado.modo_oscuro = !estado.modo_oscuro,
        Mensaje::DispositivosDetectados(d) => estado.dispositivos = Some(d),
    }
}

pub fn subscription(estado: &Estado) -> Subscription<Mensaje> {
    if estado.dispositivos.is_some() {
        Subscription::none()
    } else {
        hilos::trabajo(1, dispositivos::detectar)
            .map(envolver_dispositivos)
    }
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
        otra => Element::from(
            column![
                text(otra.titulo()).size(20.0).color(color(t.texto)),
                text("(pantalla en construcción)")
                    .size(12.0)
                    .color(color(t.texto_secundario)),
            ]
            .spacing(6.0),
        ),
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

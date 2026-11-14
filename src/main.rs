mod nucleo;
mod ui;

use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length, Padding};

use ui::tema::Tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pagina {
    Inicio,
    Imprimir,
    Escanear,
    Herramientas,
}

impl Pagina {
    fn titulo(self) -> &'static str {
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
}

#[derive(Debug)]
pub struct Estado {
    pub modo_oscuro: bool,
    pub pagina: Pagina,
}

fn boot() -> Estado {
    Estado {
        modo_oscuro: false,
        pagina: Pagina::Inicio,
    }
}

fn update(estado: &mut Estado, mensaje: Mensaje) {
    match mensaje {
        Mensaje::Ir(p) => estado.pagina = p,
        Mensaje::AlternarTema => estado.modo_oscuro = !estado.modo_oscuro,
    }
}

fn color(hex: u32) -> Color {
    Color::from_rgb8(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
}

fn view(estado: &Estado) -> Element<'_, Mensaje> {
    let t = if estado.modo_oscuro {
        Tokens::oscuro()
    } else {
        Tokens::claro()
    };

    fn boton_nav<'a>(etiqueta: &'a str, pagina: Pagina, activa: bool) -> button::Button<'a, Mensaje> {
        let fondo = if activa { color(0x3A46D6) } else { Color::TRANSPARENT };
        let letra = if activa { color(0xCDD3FF) } else { color(0xA6ACC2) };
        button(
            container(text(etiqueta).size(14).color(letra))
                .center_x(Length::Fill)
                .padding(6.0),
        )
        .width(Length::Fill)
        .style(move |_tema, _estado| button::Style {
            background: Some(fondo.into()),
            text_color: letra,
            ..button::Style::default()
        })
        .on_press(Mensaje::Ir(pagina))
    }

    let lateral = container(
        column![
            text("PrintDesk").size(15).color(color(0xF0F2FA)),
            boton_nav("Inicio", Pagina::Inicio, estado.pagina == Pagina::Inicio),
            boton_nav(
                "Imprimir",
                Pagina::Imprimir,
                estado.pagina == Pagina::Imprimir
            ),
            boton_nav(
                "Escanear",
                Pagina::Escanear,
                estado.pagina == Pagina::Escanear
            ),
            boton_nav(
                "Herramientas",
                Pagina::Herramientas,
                estado.pagina == Pagina::Herramientas
            ),
            button(text(if estado.modo_oscuro { "Modo claro" } else { "Modo oscuro" })
                .size(13)
                .color(color(0xA6ACC2)))
                .on_press(Mensaje::AlternarTema),
        ]
        .spacing(6)
        .padding(12),
    )
    .width(Length::Fixed(150.0))
    .height(Length::Fill)
    .style(move |_tema| container::Style {
        background: Some(color(t.lado).into()),
        ..container::Style::default()
    });

    let contenido = container(
        column![
            text(estado.pagina.titulo()).size(22).color(color(t.texto)),
            text("(pantalla en construcción)")
                .size(13)
                .color(color(t.texto_secundario)),
        ]
        .spacing(8),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding::new(20.0))
    .style(move |_tema| container::Style {
        background: Some(color(t.fondo).into()),
        ..container::Style::default()
    });

    row![lateral, contenido].into()
}

fn main() -> iced::Result {
    iced::application(boot, update, view)
        .title("PrintDesk")
        .window_size(iced::Size::new(1080.0, 720.0))
        .run()
}

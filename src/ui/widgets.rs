//! Piezas visuales compartidas, siempre desde los tokens del tema.

use iced::widget::{button, container, text};
use iced::{Color, Element, Length};

use super::tema::Tokens;

pub fn color(hex: u32) -> Color {
    Color::from_rgb8(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
}

/// Tarjeta: superficie con borde suave y radios del spec.
pub fn tarjeta<'a, M: 'a>(
    t: &Tokens,
    contenido: impl Into<Element<'a, M>>,
) -> container::Container<'a, M> {
    let (superficie, borde) = (color(t.superficie), color(t.borde));
    container(contenido)
        .padding(14.0)
        .width(Length::Fill)
        .style(move |_t| container::Style {
            background: Some(superficie.into()),
            border: iced::border::rounded(12.0).color(borde),
            ..container::Style::default()
        })
}

/// Botón primario (índigo, texto blanco).
pub fn boton_primario<'a, M: Clone + 'a>(
    t: &Tokens,
    etiqueta: &str,
    al_pulsar: M,
) -> button::Button<'a, M> {
    let (primario, primario_h) = (color(t.primario), color(t.primario_h));
    button(text(etiqueta.to_string()).size(13.0).color(Color::WHITE))
        .padding([9.0, 16.0])
        .style(move |_t, estado| button::Style {
            background: Some(
                if estado == button::Status::Hovered {
                    primario_h
                } else {
                    primario
                }
                .into(),
            ),
            border: iced::border::rounded(10.0),
            text_color: Color::WHITE,
            ..button::Style::default()
        })
        .on_press(al_pulsar)
}

/// Botón secundario (superficie con borde; primario al pasar el ratón).
pub fn boton_secundario<'a, M: Clone + 'a>(
    t: &Tokens,
    etiqueta: &str,
    al_pulsar: M,
) -> button::Button<'a, M> {
    let (borde, texto, primario) = (color(t.borde), color(t.texto), color(t.primario));
    button(text(etiqueta.to_string()).size(13.0).color(texto))
        .padding([9.0, 16.0])
        .style(move |_t, estado| button::Style {
            background: None,
            border: iced::border::rounded(10.0).color(if estado == button::Status::Hovered {
                primario
            } else {
                borde
            }),
            text_color: texto,
            ..button::Style::default()
        })
        .on_press(al_pulsar)
}

/// Chip de estado: punto verde si `ok`, ámbar si no.
pub fn chip<'a, M: 'a>(t: &Tokens, etiqueta: &str, ok: bool) -> container::Container<'a, M> {
    let (punto, texto_chip) = if ok {
        (color(t.correcto), color(t.texto))
    } else {
        (color(0xD98A2B), color(t.texto_secundario))
    };
    let (lienzo, borde) = (color(t.lienzo), color(t.borde));
    container(
        iced::widget::row![
            container(text::Text::new("").size(8.0))
                .width(8.0)
                .height(8.0)
                .style(move |_t| container::Style {
                    background: Some(punto.into()),
                    border: iced::border::rounded(999.0),
                    ..container::Style::default()
                }),
            text(etiqueta.to_string()).size(11.0).color(texto_chip),
        ]
        .spacing(6.0)
        .align_y(iced::Alignment::Center),
    )
    .padding([4.0, 10.0])
    .style(move |_t| container::Style {
        background: Some(lienzo.into()),
        border: iced::border::rounded(999.0).color(borde),
        ..container::Style::default()
    })
}

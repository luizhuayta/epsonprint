//! Un solo trazo para navegación y acciones: SVG locales, sin fuentes de iconos.
use iced::{Color, widget::svg};

pub fn icono(nombre: &str, color: Color) -> svg::Svg<'static> {
    let trazos = match nombre {
        "inicio" => "<path d='m3 10 9-7 9 7M5 9v12h14V9'/>",
        "escanear" => "<path d='M4 8V4h4m8 0h4v4m0 8v4h-4m-8 0H4v-4M7 12h10'/>",
        "herramientas" => {
            "<path d='M14 5a5 5 0 0 0-6 6l-5 6 4 4 6-6a5 5 0 0 0 6-6l-4 3-3-3 3-4Z'/>"
        }
        "tema" => "<path d='M21 13A9 9 0 1 1 11 3a7 7 0 0 0 10 10Z'/>",
        _ => {
            "<path d='M6 9V3h12v6M6 18H3V9h18v9h-3'/><rect x='6' y='14' width='12' height='7' rx='1'/>"
        }
    };
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><g fill='none' stroke='#{:02x}{:02x}{:02x}' stroke-width='1.75' stroke-linecap='round' stroke-linejoin='round'>{trazos}</g></svg>",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    );
    svg::Svg::new(svg::Handle::from_memory(svg.into_bytes()))
        .width(20)
        .height(20)
}

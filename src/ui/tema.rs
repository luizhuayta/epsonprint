//! Tokens visuales de PrintDesk (spec: identidad índigo, sidebar oscura).
//! Colores como 0xRRGGBB; a `iced::Color` en la capa de UI.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tokens {
    pub fondo: u32,
    pub superficie: u32,
    pub lado: u32, // barra lateral: oscura en ambos modos
    pub borde: u32,
    pub texto: u32,
    pub texto_secundario: u32,
    pub primario: u32,
    pub primario_h: u32, // primario al pasar el ratón
    pub lienzo: u32,     // fondo del visor de documentos
    pub correcto: u32,
}

impl Tokens {
    pub fn claro() -> Self {
        Self {
            fondo: 0xF5F6FA,
            superficie: 0xFFFFFF,
            lado: 0x1B1E2B,
            borde: 0xE4E7F0,
            texto: 0x1E2230,
            texto_secundario: 0x697086,
            primario: 0x4C5BF5,
            primario_h: 0x3D49D6,
            lienzo: 0xEDEFF6,
            correcto: 0x27A06A,
        }
    }

    pub fn oscuro() -> Self {
        Self {
            fondo: 0x15171F,
            superficie: 0x1E212B,
            lado: 0x101221,
            borde: 0x2A2E3F,
            texto: 0xE9EBF5,
            texto_secundario: 0x9AA1B9,
            primario: 0x6474FF,
            primario_h: 0x7B87FF,
            lienzo: 0x11131B,
            correcto: 0x27A06A,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tema_oscuro_difiere() {
        let c = Tokens::claro();
        let o = Tokens::oscuro();
        assert_ne!(c.fondo, o.fondo);
        assert_ne!(c.superficie, o.superficie);
        assert_ne!(c.texto, o.texto);
        assert_ne!(c.borde, o.borde);
    }

    #[test]
    fn primarios_del_spec() {
        assert_eq!(Tokens::claro().primario, 0x4C5BF5);
        assert_eq!(Tokens::oscuro().primario, 0x6474FF);
    }

    #[test]
    fn sidebar_oscura_en_ambos_modos() {
        let c = Tokens::claro();
        let o = Tokens::oscuro();
        // Oscura siempre: muy por debajo en luminosidad que el fondo claro.
        assert!(c.lado < 0x30_00_00 && o.lado < 0x30_00_00);
    }
}

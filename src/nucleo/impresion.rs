//! Opciones de impresión, parseo de rango, vista previa y envío a CUPS.
//!
//! Ruling (2026-11-14): v1 imprime SIEMPRE componiendo hojas (exprim `impose`
//! + `compose`) y las envía con `lp`. Así la vista previa y lo impreso son
//! exactamente lo mismo, y «Calidad» (`print-quality`), que exprim 0.4 no puede
//! transportar en `SpoolJob`, llega de verdad a la impresora.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use exprim::print::spool::{PrinterId, Spooler};
use exprim::print::{
    ComposeOptions, Margins, Orientation, PageRange, PaperSize, PrintJob, PrintOptions, Scaling,
    compose_sheet, impose, preview_sheets, PreviewOptions,
};
use exprim::RenderSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Papel {
    A4,
    Letter,
    Foto10x15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientacion {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorImp {
    Color,
    BlancoNegro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calidad {
    Borrador,
    Normal,
    Alta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Margenes {
    Estrechos,
    Normales,
    Amplios,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Escala {
    Ajustar,
    Real,
    Porcentaje(f32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rango {
    Todas,
    Paginas(Vec<(u32, u32)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorRango {
    Vacio,
    NumeroInvalido,
    Invertido,
    FueraDeRango,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpcionesImpresion {
    pub papel: Papel,
    pub orientacion: Orientacion,
    pub copias: u32,
    pub color: ColorImp,
    pub calidad: Calidad,
    pub margenes: Margenes,
    pub escala: Escala,
    pub rango: Rango,
}

impl OpcionesImpresion {
    /// A4, vertical, 1 copia, color, normal, normales, ajustar, todas.
    pub fn predeterminadas() -> Self {
        Self {
            papel: Papel::A4,
            orientacion: Orientacion::Vertical,
            copias: 1,
            color: ColorImp::Color,
            calidad: Calidad::Normal,
            margenes: Margenes::Normales,
            escala: Escala::Ajustar,
            rango: Rango::Todas,
        }
    }

    /// Traducción a las opciones de exprim (geometría, rango, copias, color).
    pub fn a_print_options(&self) -> PrintOptions {
        PrintOptions {
            paper: match self.papel {
                Papel::A4 => PaperSize::A4,
                Papel::Letter => PaperSize::Letter,
                // 10×15 cm en puntos (1 pt = 1/72"): 283.46 × 425.20.
                Papel::Foto10x15 => PaperSize::Custom {
                    width: 283.46,
                    height: 425.20,
                },
            },
            orientation: match self.orientacion {
                Orientacion::Vertical => Orientation::Portrait,
                Orientacion::Horizontal => Orientation::Landscape,
            },
            margins: Margins::uniform(match self.margenes {
                Margenes::Estrechos => 18.0,
                Margenes::Normales => 36.0,
                Margenes::Amplios => 54.0,
            }),
            scaling: match self.escala {
                Escala::Ajustar => Scaling::FitToPage,
                Escala::Real => Scaling::ActualSize,
                Escala::Porcentaje(p) => Scaling::Percent(f64::from(p)),
            },
            range: match &self.rango {
                Rango::Todas => PageRange::All,
                Rango::Paginas(v) => PageRange::Spans(v.clone()),
            },
            copies: self.copias.clamp(1, 999) as u16,
            grayscale: self.color == ColorImp::BlancoNegro,
            ..PrintOptions::default()
        }
    }
}

/// `"todas"` ⇒ Todas; `"2-5,8"` ⇒ Paginas([(2,5),(8,8)]).
/// Reglas: enteros ≥ 1, inicio ≤ fin, dentro de `total`, sin basura.
pub fn parsear_rango(texto: &str, total: u32) -> Result<Rango, ErrorRango> {
    let limpio = texto.trim();
    if limpio.is_empty() {
        return Err(ErrorRango::Vacio);
    }
    let minusculas = limpio.to_lowercase();
    if minusculas == "todas" || minusculas == "all" {
        return Ok(Rango::Todas);
    }
    let mut tramos = Vec::new();
    for parte in limpio.split(',') {
        let parte = parte.trim();
        let (a, b) = parte.split_once('-').unwrap_or((parte, parte));
        let desde: u32 = a.trim().parse().map_err(|_| ErrorRango::NumeroInvalido)?;
        let hasta: u32 = b.trim().parse().map_err(|_| ErrorRango::NumeroInvalido)?;
        if desde == 0 || hasta == 0 || hasta > total {
            return Err(ErrorRango::FueraDeRango);
        }
        if desde > hasta {
            return Err(ErrorRango::Invertido);
        }
        tramos.push((desde, hasta));
    }
    Ok(Rango::Paginas(tramos))
}

#[derive(Debug)]
pub enum ErrorImpresion {
    Abrir(exprim::ReadError),
    Imprimir(exprim::print::PrintError),
    Io(std::io::Error),
    Lp(String),
}

impl From<exprim::ReadError> for ErrorImpresion {
    fn from(e: exprim::ReadError) -> Self {
        ErrorImpresion::Abrir(e)
    }
}

impl From<exprim::print::PrintError> for ErrorImpresion {
    fn from(e: exprim::print::PrintError) -> Self {
        ErrorImpresion::Imprimir(e)
    }
}

impl From<std::io::Error> for ErrorImpresion {
    fn from(e: std::io::Error) -> Self {
        ErrorImpresion::Io(e)
    }
}

impl std::fmt::Display for ErrorImpresion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorImpresion::Abrir(e) => write!(f, "el documento no se pudo abrir ({e})"),
            ErrorImpresion::Imprimir(e) => write!(f, "error de impresión: {e}"),
            ErrorImpresion::Io(e) => write!(f, "error de disco: {e}"),
            ErrorImpresion::Lp(e) => write!(f, "CUPS rechazó el trabajo: {e}"),
        }
    }
}

/// Vista previa fiel: PNG por hoja, tal como saldrá en papel.
pub fn previsualizar(pdf: &[u8], opts: &OpcionesImpresion) -> Result<Vec<Vec<u8>>, ErrorImpresion> {
    let bytes: Arc<[u8]> = pdf.into();
    let session = RenderSession::open(bytes, None)?;
    let job = PrintJob::from_session(&session, opts.a_print_options())?;
    let sheets = impose(&job, Margins::ZERO)?;
    let mut hojas = Vec::with_capacity(sheets.len());
    for png in preview_sheets(&session, &sheets, PreviewOptions::default()) {
        hojas.push(png?);
    }
    Ok(hojas)
}

/// Envío genérico a través de un spooler exprim (tests: `FileSpooler`).
pub fn imprimir(
    spooler: &dyn Spooler,
    impresora: &str,
    doc: &[u8],
    titulo: &str,
    opts: &OpcionesImpresion,
) -> Result<String, ErrorImpresion> {
    let bytes: Arc<[u8]> = doc.into();
    let impresora = PrinterId(String::from(impresora));
    let trabajo = exprim::print::print_document(
        spooler,
        &impresora,
        titulo,
        bytes,
        &opts.a_print_options(),
    )?;
    Ok(trabajo.id.0)
}

/// Envío real a CUPS componiendo hojas nosotros: control total de opciones
/// (`print-quality` incluida). `binario_lp` inyectable para tests.
pub fn imprimir_cups(
    impresora: &str,
    doc: &[u8],
    titulo: &str,
    opts: &OpcionesImpresion,
    binario_lp: &str,
    dir_tmp: &Path,
) -> Result<String, ErrorImpresion> {
    let bytes: Arc<[u8]> = doc.into();
    let session = RenderSession::open(bytes, None)?;
    let job = PrintJob::from_session(&session, opts.a_print_options())?;
    let sheets = impose(&job, Margins::ZERO)?;

    let mut archivos = Vec::with_capacity(sheets.len());
    for (n, sheet) in sheets.iter().enumerate() {
        let raster = compose_sheet(&session, sheet, &ComposeOptions::default())?;
        let ruta = dir_tmp.join(format!("hoja-{n:04}.png"));
        escribir_png(&ruta, &raster)?;
        archivos.push(ruta);
    }

    let mut cmd = Command::new(binario_lp);
    cmd.arg("-d").arg(impresora);
    if !titulo.is_empty() {
        cmd.arg("-t").arg(titulo);
    }
    cmd.arg("-o").arg(format!("print-quality={}", calidad_ipp(opts.calidad)));
    cmd.arg("-o").arg(format!("print-color-mode={}", color_ipp(opts.color)));
    cmd.arg("-o").arg("sides=one-sided");
    cmd.arg("-o").arg(format!("media={}", media_ipp(opts.papel)));
    if opts.copias > 1 {
        cmd.arg("-n").arg(opts.copias.to_string());
        cmd.arg("-o").arg("Collate=True");
    }
    for archivo in &archivos {
        cmd.arg(archivo);
    }
    let salida = cmd.output().map_err(|e| ErrorImpresion::Lp(e.to_string()))?;
    if !salida.status.success() {
        return Err(ErrorImpresion::Lp(
            String::from_utf8_lossy(&salida.stderr).trim().to_string(),
        ));
    }
    let stdout = String::from_utf8_lossy(&salida.stdout);
    id_de_trabajo(&stdout).ok_or_else(|| ErrorImpresion::Lp("sin id de trabajo".into()))
}

fn calidad_ipp(c: Calidad) -> u8 {
    match c {
        Calidad::Borrador => 3,
        Calidad::Normal => 4,
        Calidad::Alta => 5,
    }
}

fn color_ipp(c: ColorImp) -> &'static str {
    match c {
        ColorImp::Color => "color",
        ColorImp::BlancoNegro => "monochrome",
    }
}

fn media_ipp(p: Papel) -> String {
    match p {
        Papel::A4 => "A4".into(),
        Papel::Letter => "Letter".into(),
        Papel::Foto10x15 => "Custom.100x150mm".into(),
    }
}

/// `request id is Epson_L375-42 (1 file(s))` ⇒ `Epson_L375-42`.
/// Estructural: el último token con forma `<nombre>-<dígitos>`; el resto de la
/// frase está localizado (esta máquina responde en español), el id no.
fn id_de_trabajo(stdout: &str) -> Option<String> {
    stdout
        .split_whitespace()
        .filter(|t| t.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '(' || c == ')'))
        .rfind(|t| {
            matches!(t.rsplit_once('-'), Some((nombre, digitos)) if !nombre.is_empty()
                && !digitos.is_empty()
                && digitos.chars().all(|c| c.is_ascii_digit()))
        })
        .map(|t| t.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-').to_string())
}

fn escribir_png(ruta: &Path, raster: &exprim::print::SheetRaster) -> std::io::Result<()> {
    use image::codecs::png::PngEncoder;
    use image::{ExtendedColorType, ImageEncoder as _};

    let color = if raster.channels == 1 {
        ExtendedColorType::L8
    } else {
        ExtendedColorType::Rgb8
    };
    PngEncoder::new(std::fs::File::create(ruta)?)
        .write_image(&raster.pixels, raster.width, raster.height, color)
        .map_err(std::io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// PDF de 3 páginas A4 vacío, a mano: exprim reconstruye el xref roto.
    fn pdf_3_paginas() -> Vec<u8> {
        let paginas: String = (0..3)
            .map(|i| {
                format!(
                    "{} 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 6 0 R >> endobj\n",
                    i + 3
                )
            })
            .collect();
        format!(
            "%PDF-1.4\n\
             1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n\
             2 0 obj << /Type /Pages /Kids [3 0 R 4 0 R 5 0 R] /Count 3 >> endobj\n\
             {paginas}\
             6 0 obj << /Length 13 >> stream\n0 0 100 100 re f\nendstream endobj\n\
             trailer << /Size 7 /Root 1 0 R >>\n\
             %%EOF\n"
        )
        .into_bytes()
    }

    #[test]
    fn rango_todas() {
        assert_eq!(parsear_rango("todas", 8).unwrap(), Rango::Todas);
    }

    #[test]
    fn rango_spans() {
        assert_eq!(
            parsear_rango("2-5,8", 8).unwrap(),
            Rango::Paginas(vec![(2, 5), (8, 8)])
        );
        assert_eq!(
            parsear_rango("3", 8).unwrap(),
            Rango::Paginas(vec![(3, 3)])
        );
    }

    #[test]
    fn rango_errores() {
        assert_eq!(parsear_rango("0", 8), Err(ErrorRango::FueraDeRango));
        assert_eq!(parsear_rango("5-2", 8), Err(ErrorRango::Invertido));
        assert_eq!(parsear_rango("99", 8), Err(ErrorRango::FueraDeRango));
        assert_eq!(parsear_rango("a", 8), Err(ErrorRango::NumeroInvalido));
        assert_eq!(parsear_rango("", 8), Err(ErrorRango::Vacio));
    }

    #[test]
    fn preview_3_hojas_y_cabecera_png() {
        let hojas = previsualizar(&pdf_3_paginas(), &OpcionesImpresion::predeterminadas())
            .expect("preview");
        assert_eq!(hojas.len(), 3);
        assert!(hojas[0].starts_with(b"\x89PNG"));
    }

    #[test]
    fn preview_rango_2_3_dos_hojas() {
        let mut opts = OpcionesImpresion::predeterminadas();
        opts.rango = Rango::Paginas(vec![(2, 3)]);
        let hojas = previsualizar(&pdf_3_paginas(), &opts).expect("preview");
        assert_eq!(hojas.len(), 2);
    }

    #[test]
    fn pdf_roto_es_error_no_panico() {
        let err = previsualizar(b"%PDF-roto", &OpcionesImpresion::predeterminadas());
        assert!(err.is_err());
    }

    #[test]
    fn imprimir_con_file_spooler_devuelve_id() {
        let dir = tempfile::tempdir().unwrap();
        let spooler = exprim::print::spool::file::FileSpooler::new(dir.path());
        let id = imprimir(
            &spooler,
            "Fake",
            &pdf_3_paginas(),
            "test",
            &OpcionesImpresion::predeterminadas(),
        )
        .expect("imprimir");
        assert!(!id.is_empty());
    }

    /// Falso `lp`: guarda sus argumentos y responde con un id de trabajo.
    fn lp_falso(dir: &Path) -> (String, std::path::PathBuf) {
        let captura = dir.join("args.txt");
        let script = dir.join("lp-falso");
        let contenido = format!(
            "#!/bin/bash\necho \"$@\" > {}\necho \"request id is Fake-7 (1 file(s))\"\n",
            captura.display()
        );
        fs::write(&script, contenido).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        }
        (script.display().to_string(), captura)
    }

    #[test]
    fn imprimir_cups_lleva_calidad_y_media() {
        let dir = tempfile::tempdir().unwrap();
        let (lp, captura) = lp_falso(dir.path());
        let id = imprimir_cups(
            "Epson_L375",
            &pdf_3_paginas(),
            "mi doc",
            &OpcionesImpresion::predeterminadas(),
            &lp,
            dir.path(),
        )
        .expect("imprimir cups");
        assert_eq!(id, "Fake-7");
        let args = fs::read_to_string(&captura).unwrap();
        assert!(args.contains("-d Epson_L375"));
        assert!(args.contains("print-quality=4"), "calidad normal: {args}");
        assert!(args.contains("media=A4"));
        assert!(args.contains("sides=one-sided"));
    }

    #[test]
    fn imprimir_cups_borrador_y_blanco_y_negro() {
        let dir = tempfile::tempdir().unwrap();
        let (lp, captura) = lp_falso(dir.path());
        let mut opts = OpcionesImpresion::predeterminadas();
        opts.calidad = Calidad::Borrador;
        opts.color = ColorImp::BlancoNegro;
        opts.copias = 2;
        opts.escala = Escala::Porcentaje(80.0);
        imprimir_cups("Epson_L375", &pdf_3_paginas(), "t", &opts, &lp, dir.path()).unwrap();
        let args = fs::read_to_string(&captura).unwrap();
        assert!(args.contains("print-quality=3"), "borrador: {args}");
        assert!(args.contains("print-color-mode=monochrome"), "{args}");
        assert!(args.contains("-n 2"), "{args}");
    }
}

//! PDFs de salida: escaneos → un PDF, imagen → PDF de una página.
//!
//! Ruling (2026-11-14): se escribe el PDF a mano (JPEG/DCTDecode) en lugar de
//! usar printpdf 0.12: su API de segunda iteración no expone todavía una vía
//! estable para colocar imágenes, y lo que necesitamos es un caso fijo
//! (una imagen por página, ocupando toda la página). El PDF se reabre con
//! exprim en los tests: si estuviera mal, el test lo canta.


use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::imageops::{rotate180, rotate270, rotate90};
use image::{DynamicImage, ExtendedColorType, ImageEncoder as _};

/// Una página escaneada en memoria: PNG tal cual lo devuelve scanimage,
/// más la rotación acumulada en grados (0/90/180/270, horario).
#[derive(Debug, Clone)]
pub struct PaginaEscaneada {
    pub png: Vec<u8>,
    pub rotacion: u32,
}

#[derive(Debug)]
pub enum ErrorPdf {
    Decodificar(String),
    Escribir(std::io::Error),
}

impl From<std::io::Error> for ErrorPdf {
    fn from(e: std::io::Error) -> Self {
        ErrorPdf::Escribir(e)
    }
}

/// Imagen (PNG o JPG) → PDF de una página con la imagen a página completa.
/// La página mide lo que la imagen a `dpi` (atajo: 300 por defecto; al
/// imprimir con «Ajustar» —el valor por defecto— la escala no importa).
pub fn imagen_a_pdf(imagen: &[u8]) -> Result<Vec<u8>, ErrorPdf> {
    let pagina = a_jpeg_rotado(imagen, 0)?;
    Ok(pdf_de_jpegs(&[pagina], 300))
}

/// Páginas escaneadas → un único PDF en `destino`, aplicando la rotación
/// acumulada de cada página. `dpi` es la resolución a la que se escaneó.
pub fn paginas_a_pdf(
    paginas: &[PaginaEscaneada],
    destino: &Path,
    dpi: u16,
) -> Result<PathBuf, ErrorPdf> {
    let jpegs: Result<Vec<_>, _> = paginas
        .iter()
        .map(|p| a_jpeg_rotado(&p.png, p.rotacion))
        .collect();
    let bytes = pdf_de_jpegs(&jpegs?, dpi);
    std::fs::write(destino, bytes)?;
    Ok(destino.to_path_buf())
}

/// Una imagen ya en JPEG con sus dimensiones: lo que el escritor embebe.
struct PaginaJpeg {
    jpeg: Vec<u8>,
    ancho_px: u32,
    alto_px: u32,
}

/// PDF mínimo: catálogo, páginas, y por página un XObject JPEG (DCTDecode)
/// colocado con `cm` para ocuparla entera. Xref exacto.
fn pdf_de_jpegs(paginas: &[PaginaJpeg], dpi: u16) -> Vec<u8> {
    // ids: 1 catálogo, 2 páginas; por página i (base 0): 3+3i página,
    // 4+3i contenido, 5+3i imagen.
    let mut out: Vec<u8> = Vec::new();
    let mut offsets = vec![0usize; 3 + paginas.len() * 3]; // índice = id
    out.extend_from_slice(b"%PDF-1.4\n");

    offsets[1] = out.len();
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let kids: Vec<String> = (0..paginas.len())
        .map(|i| format!("{} 0 R", 3 + i * 3))
        .collect();
    offsets[2] = out.len();
    let kids = format!("{} 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n", 2, kids.join(" "), paginas.len());
    out.extend_from_slice(kids.as_bytes());

    for (i, p) in paginas.iter().enumerate() {
        let (id_pag, id_con, id_img) = (3 + i * 3, 4 + i * 3, 5 + i * 3);
        let ancho_pt = p.ancho_px as f64 * 72.0 / f64::from(dpi);
        let alto_pt = p.alto_px as f64 * 72.0 / f64::from(dpi);
        let contenido = format!("q\n{ancho_pt:.2} 0 0 {alto_pt:.2} 0 0 cm\n/Im0 Do\nQ\n");

        offsets[id_pag] = out.len();
        let pag = format!(
            "{id_pag} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {ancho_pt:.2} {alto_pt:.2}] \
             /Resources << /XObject << /Im0 {id_img} 0 R >> >> /Contents {id_con} 0 R >>\nendobj\n"
        );
        out.extend_from_slice(pag.as_bytes());

        offsets[id_con] = out.len();
        let con = format!(
            "{id_con} 0 obj\n<< /Length {} >>\nstream\n{contenido}endstream\nendobj\n",
            contenido.len()
        );
        out.extend_from_slice(con.as_bytes());

        offsets[id_img] = out.len();
        let img = format!(
            "{id_img} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} \
             /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
            p.ancho_px, p.alto_px, p.jpeg.len()
        );
        out.extend_from_slice(img.as_bytes());
        out.extend_from_slice(&p.jpeg);
        out.extend_from_slice(b"\nendstream\nendobj\n");
    }

    let total_objs = 2 + paginas.len() * 3;
    let inicio_xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", total_objs + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for id in 1..=total_objs {
        out.extend_from_slice(format!("{:010} 00000 n \n", offsets[id]).as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objs + 1,
            inicio_xref
        )
        .as_bytes(),
    );
    out
}

/// Decodifica PNG/JPEG → rota (0/90/180/270) → re-codifica JPEG.
fn a_jpeg_rotado(bytes: &[u8], rotacion: u32) -> Result<PaginaJpeg, ErrorPdf> {
    let img = image::load_from_memory(bytes).map_err(|e| ErrorPdf::Decodificar(e.to_string()))?;
    let girada = match rotacion % 360 {
        0 => img,
        90 => DynamicImage::ImageRgba8(rotate90(&img.to_rgba8())),
        180 => DynamicImage::ImageRgba8(rotate180(&img.to_rgba8())),
        270 => DynamicImage::ImageRgba8(rotate270(&img.to_rgba8())),
        _ => img,
    };
    let rgb = girada.to_rgb8(); // JPEG sin alfa
    let mut jpeg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg, 90)
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|e| ErrorPdf::Decodificar(e.to_string()))?;
    Ok(PaginaJpeg {
        jpeg,
        ancho_px: rgb.width(),
        alto_px: rgb.height(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageFormat;
    use std::io::Cursor as IoCursor;
    use std::sync::Arc;

    fn png_color(ancho: u32, alto: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(ancho, alto, image::Rgba([180, 60, 60, 255]));
        let mut buf = IoCursor::new(Vec::new());
        DynamicImage::ImageRgba8(img)
            .write_to(&mut buf, ImageFormat::Png)
            .unwrap();
        buf.into_inner()
    }

    fn abrir(pdf: &[u8]) -> exprim::RenderSession {
        exprim::RenderSession::open(Arc::from(pdf.to_vec()), None).unwrap()
    }

    #[test]
    fn imagen_a_pdf_una_pagina_con_proporcion() {
        let pdf = imagen_a_pdf(&png_color(100, 140)).unwrap();
        let session = abrir(&pdf);
        assert_eq!(session.page_count(), 1);
        let g = session.page_geometry(0).unwrap();
        let (w, h) = (g.width(), g.height());
        let (esperado_w, esperado_h) = (100.0 * 72.0 / 300.0, 140.0 * 72.0 / 300.0);
        assert!((w - esperado_w).abs() < 0.6, "{w} vs {esperado_w}");
        assert!((h - esperado_h).abs() < 0.6, "{h} vs {esperado_h}");
    }

    #[test]
    fn paginas_a_pdf_aplica_rotacion() {
        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("salida.pdf");
        let paginas = vec![
            PaginaEscaneada { png: png_color(100, 140), rotacion: 0 },
            PaginaEscaneada { png: png_color(100, 140), rotacion: 90 },
            PaginaEscaneada { png: png_color(100, 140), rotacion: 270 },
        ];
        paginas_a_pdf(&paginas, &destino, 300).unwrap();
        assert!(destino.exists());
        let bytes = std::fs::read(&destino).unwrap();
        let session = abrir(&bytes);
        assert_eq!(session.page_count(), 3);
        // La 2.ª página girada 90°: dimensiones intercambiadas (140×100 px → 33.6×24 pt).
        let g2 = session.page_geometry(1).unwrap();
        let (w, h) = (g2.width(), g2.height());
        assert!((w - 33.6).abs() < 1.2 && (h - 24.0).abs() < 1.2, "rotada: {w}×{h}");
        // La 1.ª sin rotar mantiene proporción 100:140.
        let g1 = session.page_geometry(0).unwrap();
        assert!((g1.width() - 24.0).abs() < 0.6);
    }

    #[test]
    fn imagen_vacia_es_error() {
        assert!(imagen_a_pdf(b"").is_err());
    }
}

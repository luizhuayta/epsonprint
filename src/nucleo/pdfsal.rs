//! PDFs de salida: escaneos → un PDF, imagen → PDF de una página.
use image::DynamicImage;
use printpdf::{
    Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, RawImage, RawImageData, RawImageFormat,
    XObjectTransform,
};
use std::path::{Path, PathBuf};

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
    crear_pdf(
        &[PaginaEscaneada {
            png: imagen.to_vec(),
            rotacion: 0,
        }],
        300,
    )
}

/// Páginas escaneadas → un único PDF en `destino`, aplicando la rotación
/// acumulada de cada página. `dpi` es la resolución a la que se escaneó.
pub fn paginas_a_pdf(
    paginas: &[PaginaEscaneada],
    destino: &Path,
    dpi: u16,
) -> Result<PathBuf, ErrorPdf> {
    let bytes = crear_pdf(paginas, dpi)?;
    // El escaneo ya está hecho: si la carpeta de destino no existe, crearla
    // antes de escribir (perder la página sería lo inaceptable).
    if let Some(padre) = destino.parent() {
        std::fs::create_dir_all(padre)?;
    }
    std::fs::write(destino, bytes)?;
    Ok(destino.to_path_buf())
}

fn crear_pdf(paginas: &[PaginaEscaneada], dpi: u16) -> Result<Vec<u8>, ErrorPdf> {
    if paginas.is_empty() || dpi == 0 {
        return Err(ErrorPdf::Decodificar(
            "Se necesitan páginas y DPI positivos".into(),
        ));
    }
    let mut doc = PdfDocument::new("PrintDesk");
    for p in paginas {
        let img =
            image::load_from_memory(&p.png).map_err(|e| ErrorPdf::Decodificar(e.to_string()))?;
        let img = match p.rotacion % 360 {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            _ => img,
        };
        // Componer transparencia sobre papel blanco antes de guardar.
        let rgba = img.to_rgba8();
        let mut rgb = image::RgbImage::new(rgba.width(), rgba.height());
        for (destino, origen) in rgb.pixels_mut().zip(rgba.pixels()) {
            let alfa = u32::from(origen[3]);
            for canal in 0..3 {
                destino[canal] =
                    ((u32::from(origen[canal]) * alfa + 255 * (255 - alfa)) / 255) as u8;
            }
        }
        let (w, h) = (rgb.width(), rgb.height());
        let id = doc.add_image(&RawImage {
            pixels: RawImageData::U8(rgb.into_raw()),
            width: w as usize,
            height: h as usize,
            data_format: RawImageFormat::RGB8,
            tag: vec![],
        });
        doc.pages.push(PdfPage::new(
            Mm(w as f32 * 25.4 / dpi as f32),
            Mm(h as f32 * 25.4 / dpi as f32),
            vec![Op::UseXobject {
                id,
                transform: XObjectTransform {
                    dpi: Some(dpi as f32),
                    ..Default::default()
                },
            }],
        ));
    }
    Ok(doc.save(&PdfSaveOptions::default(), &mut vec![]))
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
            PaginaEscaneada {
                png: png_color(100, 140),
                rotacion: 0,
            },
            PaginaEscaneada {
                png: png_color(100, 140),
                rotacion: 90,
            },
            PaginaEscaneada {
                png: png_color(100, 140),
                rotacion: 270,
            },
        ];
        paginas_a_pdf(&paginas, &destino, 300).unwrap();
        assert!(destino.exists());
        let bytes = std::fs::read(&destino).unwrap();
        let session = abrir(&bytes);
        assert_eq!(session.page_count(), 3);
        // La 2.ª página girada 90°: dimensiones intercambiadas (140×100 px → 33.6×24 pt).
        let g2 = session.page_geometry(1).unwrap();
        let (w, h) = (g2.width(), g2.height());
        assert!(
            (w - 33.6).abs() < 1.2 && (h - 24.0).abs() < 1.2,
            "rotada: {w}×{h}"
        );
        // La 1.ª sin rotar mantiene proporción 100:140.
        let g1 = session.page_geometry(0).unwrap();
        assert!((g1.width() - 24.0).abs() < 0.6);
    }

    #[test]
    fn pdf_rechaza_sesion_vacia_y_resolucion_cero() {
        let dir = tempfile::tempdir().unwrap();
        assert!(paginas_a_pdf(&[], &dir.path().join("vacio.pdf"), 300).is_err());
        let p = PaginaEscaneada {
            png: png_color(10, 14),
            rotacion: 0,
        };
        assert!(paginas_a_pdf(&[p], &dir.path().join("cero.pdf"), 0).is_err());
    }

    #[test]
    fn imagen_vacia_es_error() {
        assert!(imagen_a_pdf(b"").is_err());
    }
}

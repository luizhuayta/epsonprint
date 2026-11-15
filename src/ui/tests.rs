use super::*;

#[test]
fn suscripcion_de_preview_conserva_identificador() {
    let mut e = boot();
    e.dispositivos = Some(Dispositivos::default());
    e.imp.documento = Some(imprimir::Documento {
        nombre: "prueba.pdf".into(),
        bytes: vec![],
        paginas: 1,
        ruta: PathBuf::from("prueba.pdf"),
    });
    e.imp.previsualizando = true;
    e.imp.id_preview = 7;
    assert!(subscription(&e).units() >= 1);
}

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
    // Documento sintético: ninguna captura del escritorio ni datos personales.
    let demo = image::RgbImage::from_fn(600, 848, |x, y| {
        let cabecera = (65..535).contains(&x) && (60..185).contains(&y);
        let linea = (65..500).contains(&x) && (250..710).contains(&y) && (y - 250) % 45 < 8;
        image::Rgb(if cabecera {
            [76, 91, 245]
        } else if linea {
            [208, 213, 228]
        } else {
            [255, 255, 255]
        })
    });
    let mut png = std::io::Cursor::new(Vec::new());
    demo.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let png = png.into_inner();
    let pdf = crate::nucleo::pdfsal::imagen_a_pdf(&png).unwrap();
    let hojas = crate::nucleo::impresion::previsualizar(
        &pdf,
        &crate::nucleo::impresion::OpcionesImpresion::predeterminadas(),
    )
    .unwrap();
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
            e.imp.documento = Some(imprimir::Documento {
                nombre: "Documento de ejemplo.pdf".into(),
                bytes: pdf.clone(),
                paginas: 1,
                ruta: PathBuf::from("ejemplo.pdf"),
            });
            e.imp.hojas = hojas
                .iter()
                .map(|png| {
                    let rgba = image::load_from_memory(png).unwrap().to_rgba8();
                    iced::widget::image::Handle::from_rgba(
                        rgba.width(),
                        rgba.height(),
                        rgba.into_raw(),
                    )
                })
                .collect();
            let captura = escanear::Captura {
                pagina: crate::nucleo::pdfsal::PaginaEscaneada {
                    png: png.clone(),
                    rotacion: 0,
                },
                visor: {
                    let rgba = image::load_from_memory(&png).unwrap().to_rgba8();
                    iced::widget::image::Handle::from_rgba(
                        rgba.width(),
                        rgba.height(),
                        rgba.into_raw(),
                    )
                },
            };
            for _ in 0..2 {
                escanear::update(
                    &mut e.scan,
                    escanear::MensajeEscanear::Lista(Ok(captura.clone())),
                );
            }
            herramientas::update(&mut e.dni, herramientas::MensajeHerramientas::Escanear);
            herramientas::update(
                &mut e.dni,
                herramientas::MensajeHerramientas::Lista(Ok(captura)),
            );
            let settings = iced::Settings {
                fonts: vec![
                    include_bytes!("../../assets/Inter-Regular.ttf")
                        .as_slice()
                        .into(),
                ],
                default_font: iced::Font::with_name("Inter"),
                ..Default::default()
            };
            let mut sim =
                iced_test::Simulator::with_size(settings, iced::Size::new(1080.0, 720.0), view(&e));
            let _ = sim.snapshot(&theme(&e)).unwrap();
            let snapshot = sim.snapshot(&theme(&e)).unwrap();
            let path = format!(
                "/tmp/opencode/printdesk-publicacion-{:?}-{}.png",
                pagina,
                if oscuro { "oscuro" } else { "claro" }
            );
            assert!(snapshot.matches_image(path).unwrap());
        }
    }
}

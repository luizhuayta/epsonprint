//! Escaneo por subproceso `scanimage` (SANE). Bloqueante por diseño: la UI lo
//! llama desde un hilo. Cancelable matando el subproceso.

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::pdfsal::PaginaEscaneada;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modo {
    Color,
    Grises,
    Linea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatoSalida {
    Pdf,
    Png,
    Jpg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpcionesEscaneo {
    pub resolucion: u32,
    pub modo: Modo,
    pub formato_salida: FormatoSalida,
}

impl OpcionesEscaneo {
    /// 300 dpi, color, PDF.
    pub fn predeterminadas() -> Self {
        Self {
            resolucion: 300,
            modo: Modo::Color,
            formato_salida: FormatoSalida::Pdf,
        }
    }
}

#[derive(Debug)]
pub enum ErrorEscaneo {
    Cancelado,
    Scanimage(String),
}

/// Argumentos de `scanimage` para una página por la placa, PNG a stdout.
pub fn args_scanimage(dispositivo: &str, o: &OpcionesEscaneo) -> Vec<String> {
    let modo = match o.modo {
        Modo::Color => "Color",
        Modo::Grises => "Gray",
        Modo::Linea => "Lineart",
    };
    vec![
        "-d".into(),
        dispositivo.into(),
        "--mode".into(),
        modo.into(),
        "--resolution".into(),
        o.resolucion.to_string(),
        "--format".into(),
        "png".into(),
    ]
}

/// Escanea una página y la devuelve como PNG en memoria.
/// `cancel` se consulta desde otro hilo; al activarse se mata el subproceso.
pub fn escanear(
    dispositivo: &str,
    o: &OpcionesEscaneo,
    cancel: Arc<AtomicBool>,
    binario: &str,
) -> Result<PaginaEscaneada, ErrorEscaneo> {
    let args = args_scanimage(dispositivo, o);
    let mut hijo = Command::new(binario)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ErrorEscaneo::Scanimage(e.to_string()))?;

    let mut cancelado = false;
    loop {
        if cancel.load(Ordering::SeqCst) {
            let _ = hijo.kill();
            cancelado = true;
            break;
        }
        match hijo.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(ErrorEscaneo::Scanimage(e.to_string())),
        }
    }
    if cancelado {
        return Err(ErrorEscaneo::Cancelado);
    }
    let salida = hijo
        .wait_with_output()
        .map_err(|e| ErrorEscaneo::Scanimage(e.to_string()))?;
    if !salida.status.success() {
        return Err(ErrorEscaneo::Scanimage(
            String::from_utf8_lossy(&salida.stderr).trim().to_string(),
        ));
    }
    Ok(PaginaEscaneada {
        png: salida.stdout,
        rotacion: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn script_falso(dir: &std::path::Path, nombre: &str, cuerpo: &str) -> String {
        let ruta = dir.join(nombre);
        std::fs::write(&ruta, format!("#!/bin/bash\n{cuerpo}\n")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&ruta, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        ruta.display().to_string()
    }

    #[test]
    fn args_correctos() {
        let args = args_scanimage("epson2:libusb:001:002", &OpcionesEscaneo::predeterminadas());
        assert_eq!(
            args,
            vec![
                "-d".to_string(),
                "epson2:libusb:001:002".to_string(),
                "--mode".to_string(),
                "Color".to_string(),
                "--resolution".to_string(),
                "300".to_string(),
                "--format".to_string(),
                "png".to_string(),
            ]
        );
    }

    #[test]
    fn args_otro_modo_y_resolucion() {
        let o = OpcionesEscaneo {
            resolucion: 600,
            modo: Modo::Grises,
            formato_salida: FormatoSalida::Png,
        };
        let args = args_scanimage("x", &o);
        assert!(args.contains(&"--mode".to_string()));
        assert!(args.windows(2).any(|w| w == ["--mode", "Gray"]));
        assert!(args.windows(2).any(|w| w == ["--resolution", "600"]));
    }

    #[test]
    fn binario_falso_devuelve_pagina() {
        let dir = tempfile::tempdir().unwrap();
        let bin = script_falso(dir.path(), "scanimage-falso", "printf 'PNG-falso'");
        let pagina = escanear("dev", &OpcionesEscaneo::predeterminadas(), Arc::new(AtomicBool::new(false)), &bin)
            .expect("escanear");
        assert_eq!(pagina.png, b"PNG-falso");
        assert_eq!(pagina.rotacion, 0);
    }

    #[test]
    fn cancelar_mata_rapido() {
        let dir = tempfile::tempdir().unwrap();
        let bin = script_falso(dir.path(), "scanimage-lento", "sleep 30");
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_clon = Arc::clone(&cancel);
        let inicio = std::time::Instant::now();
        // Cancelar a los 200 ms desde otro hilo.
        let hilo = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            cancel_clon.store(true, Ordering::SeqCst);
        });
        let res = escanear("dev", &OpcionesEscaneo::predeterminadas(), cancel, &bin);
        hilo.join().unwrap();
        assert!(matches!(res, Err(ErrorEscaneo::Cancelado)));
        assert!(inicio.elapsed() < Duration::from_secs(3), "tardó {:?}", inicio.elapsed());
    }

    #[test]
    fn binario_que_falla_es_error_claro() {
        let dir = tempfile::tempdir().unwrap();
        let bin = script_falso(dir.path(), "scanimage-error", "echo 'backend busy' >&2; exit 1");
        let res = escanear("dev", &OpcionesEscaneo::predeterminadas(), Arc::new(AtomicBool::new(false)), &bin);
        match res {
            Err(ErrorEscaneo::Scanimage(msg)) => assert!(msg.contains("backend busy"), "{msg}"),
            otro => panic!("esperaba Scanimage, llegó {otro:?}"),
        }
    }
}

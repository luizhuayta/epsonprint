//! Detección de dispositivos: impresoras (CUPS vía exprim) y escáneres (SANE vía
//! `scanimage -L`). Estados reales: si algo falla, lista vacía, nunca pánico.

use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Impresora {
    pub nombre: String,
    pub por_defecto: bool,
    pub acepta: bool,
    pub driver: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Escaner {
    pub id: String,
    pub descripcion: String,
}

#[derive(Debug, Clone, Default)]
pub struct Dispositivos {
    pub impresoras: Vec<Impresora>,
    pub escaners: Vec<Escaner>,
}

/// Impresoras del sistema. CUPS caído o sin colas ⇒ lista vacía.
pub fn impresoras() -> Vec<Impresora> {
    let spooler = exprim::print::spool::platform_spooler();
    spooler
        .printers()
        .unwrap_or_default()
        .into_iter()
        .map(|p| Impresora {
            nombre: p.id.as_str().to_string(),
            por_defecto: p.is_default,
            acepta: p.accepting_jobs,
            driver: Command::new("timeout")
                .args(["10s", "lpoptions", "-p", p.id.as_str(), "-l"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                .unwrap_or_default(),
        })
        .collect()
}

/// Escáneres SANE, sin cámaras (v4l).
pub fn escaners() -> Vec<Escaner> {
    let binario = std::env::var("PRINTDESK_SCANIMAGE").unwrap_or_else(|_| "scanimage".to_string());
    escaners_con(&binario)
}

/// Igual que `escaners` con el binario `scanimage` elegido (tests: falso).
pub fn escaners_con(binario: &str) -> Vec<Escaner> {
    let salida = Command::new("timeout")
        .args(["10s", binario, "-L"])
        .output();
    match salida {
        Ok(o) if o.status.success() => parsear_scanimage_l(&String::from_utf8_lossy(&o.stdout)),
        _ => Vec::new(),
    }
}

/// Detecta todo de una vez.
pub fn detectar() -> Dispositivos {
    Dispositivos {
        impresoras: impresoras(),
        escaners: escaners(),
    }
}

/// Parseo estructural de `scanimage -L`: `device `ID' is a DESCRIPCION`.
/// Sin palabras localizadas: la salida cambia de idioma, la estructura no.
pub fn parsear_scanimage_l(salida: &str) -> Vec<Escaner> {
    salida
        .lines()
        .filter_map(|linea| {
            let resto = linea.strip_prefix("device `")?;
            let (id, resto) = resto.split_once("' is a ")?;
            if id.starts_with("v4l:") {
                return None; // cámaras web, no escáneres
            }
            Some(Escaner {
                id: id.to_string(),
                descripcion: resto.trim().to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Salida real de esta máquina (español), capturada el 2026-11-14.
    const SALIDA_REAL: &str = "device `v4l:/dev/video0' is a Noname HP TrueVision HD Camera: HP Tru virtual device\ndevice `epson2:libusb:003:012' is a Epson PID 1105 flatbed scanner\n";

    #[test]
    fn parsea_salida_real_y_excluye_v4l() {
        let escaners = parsear_scanimage_l(SALIDA_REAL);
        assert_eq!(escaners.len(), 1);
        assert_eq!(escaners[0].id, "epson2:libusb:003:012");
        assert_eq!(escaners[0].descripcion, "Epson PID 1105 flatbed scanner");
    }

    #[test]
    fn cadena_vacia_sin_escaners() {
        assert!(parsear_scanimage_l("").is_empty());
    }

    #[test]
    fn lineas_basura_se_ignoran() {
        let salida = "cosas previas\ndevice `epson2:libusb:001:002' is a Epson XY\n";
        let escaners = parsear_scanimage_l(salida);
        assert_eq!(escaners.len(), 1);
        assert_eq!(escaners[0].id, "epson2:libusb:001:002");
    }

    #[test]
    fn scanimage_roto_devuelve_vacio() {
        assert!(escaners_con("/no/existe/scanimage").is_empty());
    }
}

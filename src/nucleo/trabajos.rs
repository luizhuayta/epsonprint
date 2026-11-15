//! Historial de trabajos recientes: JSON en `trabajos.json`, máx. 20,
//! el más nuevo primero. Un fichero corrupto no es error: lista vacía.

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct Trabajo {
    pub nombre: String,
    pub detalle: String,
    pub tipo: TipoTrabajo,
    pub fecha_unix: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoTrabajo {
    Impresion,
    Escaneo,
}

/// Forma en el fichero: `tipo` como texto para no acoplar el JSON al enum.
#[derive(Serialize, Deserialize)]
struct TrabajoGuardado {
    nombre: String,
    detalle: String,
    tipo: String,
    fecha_unix: i64,
}

impl From<&Trabajo> for TrabajoGuardado {
    fn from(t: &Trabajo) -> Self {
        Self {
            nombre: t.nombre.clone(),
            detalle: t.detalle.clone(),
            tipo: match t.tipo {
                TipoTrabajo::Impresion => "impresion",
                TipoTrabajo::Escaneo => "escaneo",
            }
            .into(),
            fecha_unix: t.fecha_unix,
        }
    }
}

impl From<TrabajoGuardado> for Trabajo {
    fn from(t: TrabajoGuardado) -> Self {
        Self {
            nombre: t.nombre,
            detalle: t.detalle,
            tipo: match t.tipo.as_str() {
                "escaneo" => TipoTrabajo::Escaneo,
                _ => TipoTrabajo::Impresion,
            },
            fecha_unix: t.fecha_unix,
        }
    }
}

#[derive(Debug)]
pub enum ErrorTrabajos {
    Escribir(std::io::Error),
    Serializar(serde_json::Error),
}

impl From<std::io::Error> for ErrorTrabajos {
    fn from(e: std::io::Error) -> Self {
        ErrorTrabajos::Escribir(e)
    }
}

impl From<serde_json::Error> for ErrorTrabajos {
    fn from(e: serde_json::Error) -> Self {
        ErrorTrabajos::Serializar(e)
    }
}

const MAXIMO: usize = 20;

fn fichero(dir: &Path) -> std::path::PathBuf {
    dir.join("trabajos.json")
}

/// Añade `trabajo` al principio y recorta a `MAXIMO`. Crea el fichero si no existe.
pub fn registrar(dir: &Path, trabajo: &Trabajo) -> Result<(), ErrorTrabajos> {
    let mut lista: Vec<_> = recientes(dir, MAXIMO)
        .iter()
        .map(TrabajoGuardado::from)
        .collect();
    lista.insert(0, TrabajoGuardado::from(trabajo));
    lista.truncate(MAXIMO);
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(fichero(dir), serde_json::to_vec_pretty(&lista)?)?;
    Ok(())
}

/// Los últimos `n` trabajos, el más nuevo primero. Fichero ausente o
/// corrupto ⇒ lista vacía.
pub fn recientes(dir: &Path, n: usize) -> Vec<Trabajo> {
    let Ok(bytes) = std::fs::read(fichero(dir)) else {
        return Vec::new();
    };
    serde_json::from_slice::<Vec<TrabajoGuardado>>(&bytes)
        .unwrap_or_default()
        .into_iter()
        .take(n)
        .map(Trabajo::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trabajo(nombre: &str, fecha: i64) -> Trabajo {
        Trabajo {
            nombre: nombre.into(),
            detalle: "3 págs".into(),
            tipo: TipoTrabajo::Impresion,
            fecha_unix: fecha,
        }
    }

    #[test]
    fn registra_y_recupera_en_orden() {
        let dir = tempfile::tempdir().unwrap();
        registrar(dir.path(), &trabajo("a", 100)).unwrap();
        registrar(dir.path(), &trabajo("b", 200)).unwrap();
        registrar(dir.path(), &trabajo("c", 300)).unwrap();
        let lista = recientes(dir.path(), 10);
        assert_eq!(lista.len(), 3);
        assert_eq!(lista[0].nombre, "c");
        assert_eq!(lista[2].nombre, "a");
    }

    #[test]
    fn se_recorta_a_veinte() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..25 {
            registrar(dir.path(), &trabajo(&format!("t{i}"), i)).unwrap();
        }
        let lista = recientes(dir.path(), 100);
        assert_eq!(lista.len(), 20);
        assert_eq!(lista[0].nombre, "t24");
        assert_eq!(lista[19].nombre, "t5");
    }

    #[test]
    fn fichero_corrupto_devuelve_vacio() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("trabajos.json"), "{no json").unwrap();
        assert!(recientes(dir.path(), 5).is_empty());
    }

    #[test]
    fn fichero_ausente_devuelve_vacio() {
        let dir = tempfile::tempdir().unwrap();
        assert!(recientes(dir.path(), 5).is_empty());
    }
}

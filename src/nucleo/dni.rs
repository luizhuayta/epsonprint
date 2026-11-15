//! Máquina de estados del asistente DNI por ambas caras.
//!
//! EsperandoAnverso → (Escanear) → EscaneandoAnverso → (PaginaLista)
//! → EsperandoReverso → (Escanear) → EscaneandoReverso → (PaginaLista)
//! → Completo. `PaginaLista` en `Completo` encadena el siguiente DNI.

use super::pdfsal::PaginaEscaneada;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasoDni {
    EsperandoAnverso,
    EscaneandoAnverso,
    EsperandoReverso,
    EscaneandoReverso,
    Completo,
}

#[derive(Debug)]
pub enum MensajeDni {
    Escanear,
    PaginaLista(PaginaEscaneada),
    Repetir,
    Fallo,
    Cancelar,
}

#[derive(Debug, Default)]
pub struct AsistenteDni {
    paso: PasoDni,
    anverso: Option<PaginaEscaneada>,
    reverso: Option<PaginaEscaneada>,
}

impl Default for PasoDni {
    fn default() -> Self {
        PasoDni::EsperandoAnverso
    }
}

impl AsistenteDni {
    pub fn nuevo() -> Self {
        Self::default()
    }

    pub fn paso(&self) -> PasoDni {
        self.paso
    }

    pub fn anverso(&self) -> Option<&PaginaEscaneada> {
        self.anverso.as_ref()
    }

    pub fn reverso(&self) -> Option<&PaginaEscaneada> {
        self.reverso.as_ref()
    }

    pub fn actualizar(&mut self, mensaje: MensajeDni) {
        match (self.paso, mensaje) {
            // Arrancar el escaneo de la cara que toque.
            (PasoDni::EsperandoAnverso, MensajeDni::Escanear) => {
                self.paso = PasoDni::EscaneandoAnverso;
            }
            (PasoDni::EsperandoReverso, MensajeDni::Escanear) => {
                self.paso = PasoDni::EscaneandoReverso;
            }
            // Página capturada.
            (PasoDni::EscaneandoAnverso, MensajeDni::PaginaLista(p)) => {
                self.anverso = Some(p);
                self.paso = PasoDni::EsperandoReverso;
            }
            (PasoDni::EscaneandoReverso, MensajeDni::PaginaLista(p)) => {
                self.reverso = Some(p);
                self.paso = PasoDni::Completo;
            }
            (PasoDni::EscaneandoAnverso, MensajeDni::Fallo) => self.paso = PasoDni::EsperandoAnverso,
            (PasoDni::EscaneandoReverso, MensajeDni::Fallo) => self.paso = PasoDni::EsperandoReverso,
            (PasoDni::Completo, MensajeDni::Repetir) => {
                self.reverso = None;
                self.paso = PasoDni::EsperandoReverso;
            }
            (PasoDni::Completo, MensajeDni::PaginaLista(_)) => {
                // Encadenar el siguiente DNI.
                self.anverso = None;
                self.reverso = None;
                self.paso = PasoDni::EsperandoAnverso;
            }
            // Repetir: descartar lo capturado de la cara actual (o todo si
            // aún no hay reverso).
            (paso, MensajeDni::Repetir) => {
                match paso {
                    PasoDni::EsperandoReverso | PasoDni::EscaneandoReverso => {
                        self.anverso = None;
                        self.reverso = None;
                        self.paso = PasoDni::EsperandoAnverso;
                    }
                    _ => {}
                }
            }
            // Cancelar: siempre al principio, limpio.
            (_, MensajeDni::Cancelar) => {
                self.anverso = None;
                self.reverso = None;
                self.paso = PasoDni::EsperandoAnverso;
            }
            // Cualquier otra combinación no cabe en el flujo: ignorar.
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pagina_de_prueba() -> PaginaEscaneada {
        PaginaEscaneada { png: vec![1, 2, 3], rotacion: 0 }
    }

    #[test]
    fn fallo_del_reverso_conserva_anverso() {
        let mut a = AsistenteDni::nuevo();
        a.actualizar(MensajeDni::Escanear);
        a.actualizar(MensajeDni::PaginaLista(pagina_de_prueba()));
        a.actualizar(MensajeDni::Escanear);
        a.actualizar(MensajeDni::Fallo);
        assert_eq!(a.paso(), PasoDni::EsperandoReverso);
        assert!(a.anverso().is_some());
    }

    #[test]
    fn repetir_reverso_completo_no_pierde_anverso() {
        let mut a = AsistenteDni::nuevo();
        for _ in 0..2 {
            a.actualizar(MensajeDni::Escanear);
            a.actualizar(MensajeDni::PaginaLista(pagina_de_prueba()));
        }
        a.actualizar(MensajeDni::Repetir);
        assert_eq!(a.paso(), PasoDni::EsperandoReverso);
        assert!(a.anverso().is_some() && a.reverso().is_none());
    }

    #[test]
    fn flujo_feliz_completo_el_dni() {
        let mut a = AsistenteDni::nuevo();
        assert_eq!(a.paso(), PasoDni::EsperandoAnverso);
        a.actualizar(MensajeDni::Escanear);
        assert_eq!(a.paso(), PasoDni::EscaneandoAnverso);
        a.actualizar(MensajeDni::PaginaLista(pagina_de_prueba()));
        assert_eq!(a.paso(), PasoDni::EsperandoReverso);
        assert!(a.anverso().is_some());
        a.actualizar(MensajeDni::Escanear);
        assert_eq!(a.paso(), PasoDni::EscaneandoReverso);
        a.actualizar(MensajeDni::PaginaLista(pagina_de_prueba()));
        assert_eq!(a.paso(), PasoDni::Completo);
        assert!(a.anverso().is_some() && a.reverso().is_some());
    }

    #[test]
    fn repetir_en_esperando_reverso_vuelve_al_principio() {
        let mut a = AsistenteDni::nuevo();
        a.actualizar(MensajeDni::Escanear);
        a.actualizar(MensajeDni::PaginaLista(pagina_de_prueba()));
        a.actualizar(MensajeDni::Repetir);
        assert_eq!(a.paso(), PasoDni::EsperandoAnverso);
        assert!(a.anverso().is_none());
    }

    #[test]
    fn cancelar_desde_cualquier_paso_reinicia() {
        for paso_intermedio in [
            MensajeDni::Escanear,
            MensajeDni::PaginaLista(pagina_de_prueba()),
        ] {
            let mut a = AsistenteDni::nuevo();
            a.actualizar(paso_intermedio);
            a.actualizar(MensajeDni::Cancelar);
            assert_eq!(a.paso(), PasoDni::EsperandoAnverso);
            assert!(a.anverso().is_none() && a.reverso().is_none());
        }
    }

    #[test]
    fn pagina_lista_inesperada_se_ignora() {
        let mut a = AsistenteDni::nuevo();
        a.actualizar(MensajeDni::PaginaLista(pagina_de_prueba()));
        assert_eq!(a.paso(), PasoDni::EsperandoAnverso);
        assert!(a.anverso().is_none());
    }

    #[test]
    fn completar_encadena_el_siguiente_dni() {
        let mut a = AsistenteDni::nuevo();
        a.actualizar(MensajeDni::Escanear);
        a.actualizar(MensajeDni::PaginaLista(pagina_de_prueba()));
        a.actualizar(MensajeDni::Escanear);
        a.actualizar(MensajeDni::PaginaLista(pagina_de_prueba()));
        assert_eq!(a.paso(), PasoDni::Completo);
        a.actualizar(MensajeDni::PaginaLista(pagina_de_prueba()));
        assert_eq!(a.paso(), PasoDni::EsperandoAnverso);
    }
}

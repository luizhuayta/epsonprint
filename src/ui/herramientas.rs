//! El asistente usa el mismo escáner y exportador que la captura multipágina.
use std::{path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use iced::{Element, Length, widget::{button, column, container, image, row, text}};
use crate::nucleo::dni::{AsistenteDni, MensajeDni, PasoDni};
use super::{Mensaje, escanear::Captura, tema::Tokens, widgets::{boton_primario, boton_secundario, tarjeta}};

#[derive(Debug)]
pub struct EstadoDni {
    pub asistente: AsistenteDni,
    pub anverso: Option<image::Handle>,
    pub reverso: Option<image::Handle>,
    pub cancel: Arc<AtomicBool>,
    pub id: u64,
    pub guardando: bool,
    pub aviso: String,
}
impl Default for EstadoDni {
    fn default() -> Self {
        Self { asistente: AsistenteDni::nuevo(), anverso: None, reverso: None,
            cancel: Arc::new(AtomicBool::new(false)), id: 0, guardando: false, aviso: String::new() }
    }
}
impl EstadoDni {
    pub fn escaneando(&self) -> bool {
        matches!(self.asistente.paso(), PasoDni::EscaneandoAnverso | PasoDni::EscaneandoReverso)
    }
}
#[derive(Debug, Clone)]
pub enum MensajeHerramientas {
    Escanear, Lista(Result<Captura, String>), Cancelar, Repetir, Nuevo, Guardar, Guardado(Result<PathBuf, String>),
}
pub fn update(e: &mut EstadoDni, m: MensajeHerramientas) {
    if e.guardando && !matches!(m, MensajeHerramientas::Guardado(_)) { return; }
    match m {
        MensajeHerramientas::Escanear if !e.escaneando() => {
            e.cancel = Arc::new(AtomicBool::new(false)); e.id += 1;
            e.asistente.actualizar(MensajeDni::Escanear); e.aviso.clear();
        }
        MensajeHerramientas::Lista(Ok(c)) => {
            match e.asistente.paso() {
                PasoDni::EscaneandoAnverso => e.anverso = Some(c.visor),
                PasoDni::EscaneandoReverso => e.reverso = Some(c.visor),
                _ => return,
            }
            e.asistente.actualizar(MensajeDni::PaginaLista(c.pagina));
        }
        MensajeHerramientas::Lista(Err(error)) => { e.asistente.actualizar(MensajeDni::Fallo); e.aviso = error; }
        MensajeHerramientas::Cancelar => e.cancel.store(true, Ordering::SeqCst),
        MensajeHerramientas::Repetir if !e.escaneando() => {
            e.asistente.actualizar(MensajeDni::Repetir);
            e.reverso = None;
            if e.asistente.anverso().is_none() { e.anverso = None; }
        }
        MensajeHerramientas::Nuevo if !e.escaneando() => {
            e.asistente.actualizar(MensajeDni::Cancelar); e.anverso = None; e.reverso = None; e.aviso.clear();
        }
        MensajeHerramientas::Guardar if e.asistente.paso() == PasoDni::Completo => { e.guardando = true; e.id += 1; }
        MensajeHerramientas::Guardado(res) => {
            e.guardando = false;
            e.aviso = match res { Ok(ruta) => format!("Guardado: {}", ruta.display()), Err(error) => error };
        }
        _ => {}
    }
}

pub fn vista<'a>(t: &Tokens, e: &'a EstadoDni) -> Element<'a, Mensaje> {
    let m = Mensaje::Herramientas;
    let paso = e.asistente.paso();
    let indicacion = match paso {
        PasoDni::EsperandoAnverso => "Coloca el anverso del DNI sobre el cristal A4.",
        PasoDni::EscaneandoAnverso | PasoDni::EscaneandoReverso => "Escaneando… No muevas el documento.",
        PasoDni::EsperandoReverso => "Gira el DNI. El anverso está conservado; escanea el reverso.",
        PasoDni::Completo => "Las dos caras están listas: guarda el PDF de dos páginas A4.",
    };
    let cara = |nombre: &str, visor: &Option<image::Handle>| -> Element<'a, Mensaje> {
        let mut c = column![text(nombre.to_string()).size(16)].spacing(8);
        if let Some(v) = visor { c = c.push(image(v.clone()).height(190)); }
        else { c = c.push(container(text("Pendiente")).height(190)); }
        tarjeta(t, c).into()
    };
    let acciones: Element<'a, Mensaje> = if e.escaneando() {
        boton_secundario(t, "Cancelar captura", m(MensajeHerramientas::Cancelar)).into()
    } else if e.guardando {
        text("Guardando PDF…").into()
    } else if paso == PasoDni::Completo {
        row![boton_primario(t, "Guardar PDF", m(MensajeHerramientas::Guardar)),
            boton_secundario(t, "Repetir reverso", m(MensajeHerramientas::Repetir)),
            boton_secundario(t, "Nuevo DNI", m(MensajeHerramientas::Nuevo))].spacing(8).into()
    } else {
        row![boton_primario(t, if paso == PasoDni::EsperandoAnverso { "Escanear anverso" } else { "Escanear reverso" }, m(MensajeHerramientas::Escanear)),
            button("Repetir anverso").on_press_maybe((paso == PasoDni::EsperandoReverso).then(|| m(MensajeHerramientas::Repetir)))].spacing(8).into()
    };
    column![text("DNI por ambas caras").size(24), text(indicacion), row![cara("1. Anverso", &e.anverso), cara("2. Reverso", &e.reverso)].spacing(12),
        acciones, text(e.aviso.clone()).size(12),
        text("Próxima fase").size(16),
        row![tarjeta(t, text("Recorte inteligente · pendiente")), tarjeta(t, text("Word y Excel · pendiente")), tarjeta(t, text("Dúplex manual · pendiente"))].spacing(8),
    ].spacing(14).width(Length::Fill).into()
}

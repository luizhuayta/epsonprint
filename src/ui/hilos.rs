//! Trabajo pesado fuera del executor de Iced: hilo de SO + suscripción que
//! entrega el resultado como mensaje. `id` deduplica: mientras la misma
//! suscripción esté viva no se relanza el trabajo.

use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use iced::Subscription;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt as _, StreamExt as _};
use iced::stream;

struct Datos<T> {
    id: u64,
    f: CajaDeTrabajo<T>,
}

/// El trabajo pendiente de lanzar, compartido con el stream de la suscripción.
type CajaDeTrabajo<T> = Arc<Mutex<Option<Box<dyn FnOnce() -> T + Send>>>>;

impl<T> Hash for Datos<T> {
    fn hash<H: Hasher>(&self, estado: &mut H) {
        self.id.hash(estado);
    }
}

/// Ejecuta `f` en un hilo del sistema y emite su resultado como mensaje.
/// Si `f` entra en pánico, la suscripción termina en silencio.
pub fn trabajo<T>(id: u64, f: impl FnOnce() -> T + Send + 'static) -> Subscription<T>
where
    T: Send + 'static,
{
    let datos = Datos {
        id,
        f: Arc::new(Mutex::new(Some(Box::new(f)))),
    };
    Subscription::run_with(datos, |d| {
        let f = Arc::clone(&d.f);
        stream::channel(1, async move |mut salida| {
            let Some(f) = f.lock().expect("suscripción de trabajo").take() else {
                return;
            };
            let (tx, mut rx) = mpsc::channel(1);
            std::thread::spawn(move || {
                let mut tx = tx;
                let _ = iced::futures::executor::block_on(tx.send(f()));
            });
            if let Some(valor) = rx.next().await {
                let _ = salida.send(valor).await;
            }
        })
    })
}

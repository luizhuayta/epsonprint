# PrintDesk

Aplicación de escritorio Linux en Rust + Iced para imprimir y escanear con CUPS y SANE. Pensada para la Epson EcoTank L375, sin modificar su configuración ni la rama `main`.

## Incluye

- Inicio con detección de colas CUPS y escáneres SANE, actualización manual y trabajos recientes que se pueden reabrir.
- Imprimir PDF, PNG y JPEG: miniaturas ocultables, visor con zoom, papel, orientación, copias, color, calidad del controlador, márgenes, escala y rango de páginas. Pulsa **Enter** para aplicar el rango y revisar la vista previa antes de imprimir.
- Escanear varias hojas: visor, selección de página, giro, eliminación y reordenación con flechas o arrastre. Guardar un PDF único o imágenes PNG/JPG. Los originales de la sesión permanecen disponibles después de guardar.
- Herramientas: asistente DNI anverso/reverso, repetir cara, cancelar captura y guardar PDF de dos páginas A4.
- Interfaz índigo con iconos SVG locales, Inter incluida (licencia en `assets/Inter-LICENSE.txt`), tema del sistema y cambio claro/oscuro.

## Requisitos

- Rust 1.100 o posterior para compilar.
- `lp`, `lpstat` y `lpoptions` (cliente CUPS), `scanimage` (SANE), `timeout` (coreutils) y `xdg-open`.
- Cola y escáner funcionando previamente en el sistema. PrintDesk no instala drivers.
- Linux con Wayland o X11 y un controlador gráfico compatible con wgpu.

## Ejecutar e instalar

Para desarrollar: `~/.cargo/bin/cargo run --locked`.

Para instalar en tu usuario: `bash instalar.sh`. Compila en release, instala el binario en `~/.local/bin/printdesk` y crea la entrada **PrintDesk** en el menú. Respeta `XDG_DATA_HOME`; no usa root ni sudo.

Escaneos: `~/Escaneos/`. Un nombre ya existente nunca se sobrescribe. Historial: `${XDG_DATA_HOME:-~/.local/share}/printdesk/trabajos.json`. Los temporales de impresión usan una carpeta privada que se elimina al terminar el envío.

## Verificación

- `~/.cargo/bin/cargo test`: lógica, edición de páginas, salida PDF, procesos falsos, cancelación y navegación Iced headless.
- `bash tests/instalador.bash`: instalador aislado con HOME de prueba, espacios e idempotencia.
- `~/.cargo/bin/cargo test capturas_pantallas -- --ignored`: genera capturas de las cuatro pantallas en claro/oscuro en `/tmp/opencode/` (requiere renderer headless).
- [Aceptación pendiente con la Epson real](docs/acta-aceptacion.md).

## Límites conocidos

- Una cola que acepta trabajos no confirma conexión física, papel ni tinta. La aplicación no inventa esas mediciones.
- En esta L375 el PPD ofrece **Normal/Alta** para papel normal, no Borrador. Se usan `MediaType`, `Ink` y `PageSize` reales, además de las opciones IPP. El resultado físico aún requiere la prueba de aceptación.
- Foto «4×6» equivale a 101,6 × 152,4 mm, el formato que anuncia el driver; no son 100 × 150 mm exactos.
- Los DPI se mantienen fijos mientras haya páginas en una sesión, para conservar su tamaño físico. Vacía la tira para cambiar la resolución.
- Iced/winit no anuncia arrastre de archivos en Wayland en esta versión. **Abrir** funciona en ambos; los eventos de archivos soltados se atienden donde el backend los proporcione (X11).
- Los trabajos se rasterizan para enviar la geometría vista en pantalla; puede aumentar el tamaño del trabajo frente al PDF original. La vista previa no simula tinta/papel ni colorimetría de la impresora.
- Aún no se han certificado impresión y escaneo físicos en esta versión.

Diseño y plan: `docs/superpowers/`. [Mejoras posteriores](docs/mejoras-futuras.md).

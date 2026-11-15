# Contribuir a PrintDesk

Gracias por ayudar. Esta primera versión está orientada a Linux con CUPS y SANE; el equipo probado es una Epson EcoTank L375. No hace falta tenerla para ejecutar las pruebas normales.

## Preparar el entorno

Instala Rust estable (1.100 o posterior) y las dependencias de compilación descritas en el [README](README.md#instalación). Trabaja en una rama y abre una pull request contra `main`.

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
bash tests/instalador.bash
cargo build --release --locked
```

Las pruebas normales no imprimen ni escanean. **No uses `cargo test -- --ignored`**: incluye pruebas físicas que consumen papel y activan un escáner real. Para capturas de interfaz, ejecuta únicamente `cargo test capturas_pantallas -- --ignored` (necesita un renderer headless compatible).

## Cambios

- Añade una prueba que reproduzca cada fallo antes de corregirlo.
- La lógica vive en `src/nucleo/`; Iced y sus mensajes, en `src/ui/`.
- No alteres la configuración de CUPS/SANE ni asumas funciones que el controlador no anuncia.
- No incluyas escaneos personales, datos de DNI, rutas privadas ni credenciales en capturas, fixtures o reportes.
- Mantén los textos de la interfaz en español y explica límites nuevos en el README.
- Las funciones de segunda fase están en [mejoras futuras](docs/mejoras-futuras.md); coméntalas en un issue antes de ampliar el alcance.

Al contribuir aceptas que tus aportes se distribuyan bajo la licencia MIT del proyecto. Los recursos de terceros conservan su licencia propia.

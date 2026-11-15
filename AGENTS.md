# PrintDesk

- Trabajar en ramas de desarrollo; no modificar ni integrar en `main` sin permiso.
- Skills/herramientas del agente: específicas del proyecto siempre que sea posible. No instalar globalmente por comodidad.
- Rust + Iced; lógica en `src/nucleo`, pantallas en `src/ui`. No sustituir CUPS/SANE ni instalar drivers.
- Ejecutar `~/.cargo/bin/cargo test`, `~/.cargo/bin/cargo fmt --check` y `bash tests/instalador.bash` antes de entregar. Las pruebas no deben imprimir ni escanear hardware real sin coordinarlo con el usuario.
- El PPD de la L375 usa `MediaType=PLAIN_NORMAL/PLAIN_HIGH`, `Ink=COLOR/MONO`, `PageSize`. No inventar dúplex, ADF ni niveles de tinta. Fixture: `tests/fixtures/l375-opciones.txt`.
- La aceptación física está en `docs/acta-aceptacion.md`. No marcarla completada por aprobar tests simulados.

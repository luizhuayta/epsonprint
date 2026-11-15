# Changelog

Todo lo notable de cada versión. Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y versionado [SemVer](https://semver.org/lang/es/).

## [0.1.0] — 2026-11-15

Primera versión: imprimir y escanear con la Epson EcoTank L375 desde una app
de escritorio Linux nativa, sin tocar la configuración del sistema.

### Añadido

- **Inicio**: detección real de la cola CUPS y del escáner SANE (botón
  «Actualizar dispositivos»), trabajos recientes reabribles y estados sin
  inventar niveles de tinta ni disponibilidad física.
- **Imprimir**: PDF, PNG y JPEG con vista previa de la geometría por hoja,
  miniaturas y panel de opciones ocultables, zoom, papel
  (A4/Letter/foto 4×6), orientación, copias, color, calidad del controlador
  real, márgenes, escala (ajustar/real/porcentaje) y rango de páginas.
- **Escanear**: sesiones multipágina con visor, selección, giro, eliminación
  y reordenación (flechas o arrastre); salida en PDF único o PNG/JPG por
  página; guardado sin sobrescribir archivos previos y con las páginas
  conservadas en pantalla tras guardar.
- **Herramientas**: asistente DNI por ambas caras con repetición de cara,
  cancelación y PDF de dos páginas A4.
- Tema claro/oscuro que sigue al sistema, iconos SVG propios, tipografía
  Inter incluida, interfaz en español, instalación a nivel de usuario sin
  root (`instalar.sh`) y aviso al cerrar con sesión sin guardar.
- Pruebas: 48 automáticas (más capturas headless y aceptación física
  «ignoradas» para no gastar papel), instalador probado y aceptación física
  completada con impresora real el 2026-11-15.

### Limitaciones conocidas

- El PPD de la L375 no ofrece calidad Borrador en papel normal.
- Arrastrar archivos no está disponible en Wayland con esta versión de Iced.
- La resolución queda fija mientras la sesión tenga páginas.

[0.1.0]: https://github.com/luizhuayta/epsonprint/releases/tag/v0.1.0

# Diseño: EpsonPrint — app de escritorio para la Epson L375

Fecha: 2026-11-14 · Estado: pendiente de aprobación · Rama: `feat/app-impresora-escaner`

## Intención y por qué

Luis hoy maneja su impresora/escáner **Epson L375** a puro comando: `scanimage` con un
acceso directo para escanear, y un agente por terminal para imprimir. Quiere **una sola
aplicación nativa de escritorio en Rust** que, al abrirla:

1. Detecte la impresora sola (sin configurar nada).
2. Pestaña **Imprimir**: cargar PDF (v1; imágenes JPG/PNG también), ver una vista previa
   real de lo que va a salir, elegir encuadre/copias/papel/dúplex, e imprimir.
3. Pestaña **Escanear**: escanear, ver lo escaneado, y un flujo guiado para documentos
   a dos caras (DNI): escanea una cara → la app espera y dice "dale la vuelta" →
   escanea la otra → todo termina en **un solo PDF**.

Éxito = no volver a abrir una terminal ni accesos directos sueltos para imprimir o
escanear.

## Metas

- Detección automática de impresoras CUPS (preseleccionada la `Epson_L375`) y del
  escáner SANE (`epson2`), con botón de refrescar.
- Imprimir PDF e imágenes (JPG/PNG) con vista previa por hoja, tamaño de papel,
  orientación, copias, dúplex, escalado (ajustar a página / tamaño real / porcentaje).
- Escanear por placa (flatbed) con modo (color/gris/línea), resolución (150/300/600),
  vista de cada página escaneada, borrar página, y guardar todo como un PDF.
- Flujo "documento a dos caras": tras cada cara la app indica girar el papel y espera
  confirmación; el orden de páginas queda anverso/reverso consecutivos.
- Interfaz en español. Un solo binario.

## No-metas (v1)

- Archivos Office (Excel/Word) — decisión de Luis: v1 solo PDF e imágenes. Llegarían
  después convirtiendo con LibreOffice por debajo.
- OCR, fax, niveles de tinta, gestión de colas de impresión, ADF.
- Multiplataforma (esto es para este Linux concreto; nada impide que funcione en otro).

## Anti-metas

- Roto si CUPS/SANE dejan de funcionar: la app se apoya en ellos, no los sustituye.
- Configurar la impresora o tocar colas del sistema: si algo está parado, se muestra el
  error, no se "arregla" solo.

## Restricciones

- Linux (Fedora) con CUPS y SANE ya funcionando: `lpstat -e` lista `Epson_L375`,
  `scanimage -L` lista `epson2:libusb:003:012`. La app **no** modifica configuración
  del sistema.
- Stack 100 % Rust, sin FFI: impresión y escaneo van por las herramientas de línea de
  comandos (`lp`, `lpstat`, `scanimage`), que ya funcionan en esta máquina.
- Rust 1.100 ya instalado (rustup, perfil mínimo, solo usuario).
- Todo el trabajo en esta rama; `main` no se toca.

## Enfoques considerados

| | Enfoque | Cambio | Por qué no (o sí) |
|---|---|---|---|
| **A ✅** | Rust puro: **egui** (ventana) + **exprim** (PDF: abrir, previsualizar, imprimir vía CUPS) + **scanimage** (escáner) | Sin FFI; exprim trae papel/márgenes/dúplex/escala, vista previa para GUI y un `FileSpooler` para tests. Escogido. | Riesgo: exprim v0.4 (joven). Mitigación: versión fijada; plan B = `lp` directo + `pdftoppm`, ambos presentes. |
| B | Bindings nativos: `cups_rs` + `sane-sys` + `pdfium-render` | FFI a librerías C. | Más frágil de compilar, más código, cero beneficio para el usuario. |
| C | Tauri (Rust + UI web) | UI moderna rápida. | Arrastra WebView + JavaScript; más piezas; menos "nativo". |

**Decisión de Luis:** enfoque A (elegido en conversación).

## Diseño

### Piezas

```
crates/… no: un solo binario, módulos planos en src/
src/
  main.rs      – app eframe, pestañas «Imprimir» y «Escanear», hilo de fondo
  device.rs    – detectar impresoras (lpstat) y escáner (scanimage -L), con caché
  print_tab.rs – cargar archivo, opciones, vista previa de hojas, botón Imprimir
  scan_tab.rs  – controles de escaneo, lista de páginas, flujo dos caras, guardar PDF
  scan.rs      – envolver scanimage (subproceso → PNG en disco + RGBA para la UI)
  pdfout.rs    – imágenes escaneadas → un PDF; imagen suelta → PDF de 1 página
                 (para que las imágenes usen la misma tubería de impresión que los PDF)
```

- Carga de archivos: botón «Abrir» + arrastrar y soltar sobre la ventana.
- Vista previa (Imprimir): miniaturas de hojas con `exprim::print::preview_sheets`,
  recalculada al cambiar opciones; con los márgenes reales de la impresora cuando
  estén disponibles (`Spooler::capabilities` + `plan_sheets`).
- Impresión: `exprim::print::print_document` contra la impresora elegida.
- Escaneo: `scanimage -d <dev> --mode <m> --resolution <dpi> --format=png` en un hilo
  aparte (un escaneo a 300 dpi tarda decenas de segundos; la UI no se congela).
  Si `scanimage --help` del dispositivo revela fuente ADF, se añade como opción;
  si no, solo placa (lo que hoy reporta: flatbed).
- Flujo dos caras: máquina de estados simple — tras cada escaneo en modo dos caras la
  app alterna el aviso «Coloca la otra cara y pulsa Escanear» / «Página lista».
  Las páginas se numeran 1, 2, 3… en orden de escaneo; borrar página permite repetir
  una cara.
- Guardar escaneo: PDF único en `~/Escaneos/AAAA-MM-DD_HHMM.pdf` (carpeta creada si
  no existe) con botón «Abrir carpeta».
- Hilos: escaneo e impresión corren en `std::thread` con canal de resultados hacia la
  UI; cancelación del escaneo matando el subproceso.

### Dependencias (todas puras Rust)

- `eframe`/`egui` — GUI.
- `exprim` 0.4, feature `print` — abrir/render PDF, opciones de impresión, CUPS.
- `printpdf` (u otra minimal) — escribir PDFs (escaneos; envoltorio de imágenes).
- `image` — decodificar PNG/JPG para la UI (egui solo come RGBA).

### Tests

- `FileSpooler` de exprim para probar la tubería de impresión sin impresora.
- Máquina de estados del flujo dos caras: unitaria.
- `pdfout`: el PDF generado se reabre con exprim y tiene las páginas esperadas.
- Parseo de `lpstat -e` / `scanimage -L`: unitario con salidas de ejemplo fijadas.
- Aceptación final manual de Luis con la L375 real (imprimir un PDF, escanear un DNI).

### Riesgos

- **exprim v0.4 publicado hace días**: se fija la versión exacta; si su tubería de
  impresión falla, el relevo es llamar a `lp` directamente (la vista previa con
  `pdftoppm`). El diseño aísla la impresión en `print_tab.rs`/exprim para ese relevo.
- **scanimage bloquea** durante el escaneo: ya contemplado con hilo aparte.
- **egui es austero**: es una herramienta, no una joya visual; aceptado.

## Queda a criterio del constructor

- Distribución exacta de widgets, tamaños y textos de la UI.
- Detalles del envoltorio imagen→PDF (márgenes por defecto, DPI asumido).
- Cómo mostrar el progreso del escaneo (spinner + texto basta).

## Notas de proceso

- Skills: brainstorming + ponytail (activo) + TDD en la implementación. No hace falta
  instalar ninguna skill extra de Rust; si aparece una necesidad concreta, se
  instalará **a nivel proyecto**, nunca global (regla de Luis).
- Entrega académica/estilo: este documento es la fuente de verdad del diseño.

# Plan de implementación: PrintDesk v1

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Meta:** PrintDesk, app de escritorio Iced en español para imprimir (PDF/imágenes con vista previa fiel) y escanear (varias páginas → un PDF, asistente DNI dos caras) con la Epson L375.

**Arquitectura:** núcleo sin UI (`src/nucleo/*`, toda la lógica y los tests) + capa Iced (`src/ui/*`) que solo dibuja y manda mensajes. Impresión: exprim 0.4 (`print`) contra CUPS. Escaneo: subproceso `scanimage` inyectable. PDFs de salida: printpdf.

**Stack:** Rust 1.100 · iced 0.14 (features `image`, `svg`) · exprim 0.4 (`print`) · printpdf · image · rfd · dirs · chrono · dev: tempfile.

**Spec:** `docs/superpowers/specs/2026-11-14-epsonprint-design.md` (aprobado; leer junto a este plan).

## Restricciones globales

- Nombre del paquete y binario: `printdesk`. Edición 2024 de Rust.
- Toda la interfaz en español, con los tokens visuales del spec (índigo `#4C5BF5`/`#6474FF`, sidebar oscura siempre, radios 12/10/999).
- Papel por defecto **A4**; escaneos en `~/Escaneos/`; historial en `~/.local/share/printdesk/`.
- Sin FFI: CUPS/SANE solo vía `lp`/`lpstat`/`scanimage` (a través de exprim o `std::process`).
- Ningún test requiere hardware: todo con fixtures, spooler de ficheros o binario `scanimage` falso (`PRINTDESK_SCANIMAGE`). Pruebas reales solo en la Tarea 13.
- Nunca tocar `main`; commits conventional (`feat:`, `test:`, `docs:`), mensaje en español.
- Deps permitidas: las del stack, nada más sin anotarlo en el plan.

## Review Focus (entradas que muerden y su test ancla)

1. **PDF dañado o con contraseña** → error tipado y mensaje claro, sin pánico. Test: Tarea 3 (`abre pdf dañado → Err, no panic`).
2. **Rango de páginas mal formado** (`0`, `5-2`, `99` con 8 páginas) → `Err(RangoInvalido)`. Test: Tarea 3, `parsear_rango`.
3. **CUPS caído o sin impresoras** (`lpstat` falla) → lista vacía + estado visible, no pánico. Test: Tarea 2 (`impresoras_con_lpstat_roto`).
4. **`scanimage -L` en español y con cámaras v4l** → solo escáneres reales (excluir `v4l:`). Test: Tarea 2 con la salida real de esta máquina (fixture abajo).
5. **Escaneo cancelado a mitad** (kill del subproceso) → `Err(Cancelado)` y el asistente vuelve a poder escanear. Test: Tarea 5 con binario falso que duerme.

---

### Task 1: Andamio — ventana, tema y navegación

**Files:**
- Create: `Cargo.toml`, `src/main.rs`, `src/ui/mod.rs`, `src/ui/tema.rs`
- Test: `src/ui/tema.rs` (unitarios inline)

**Interfaces:**
- Produces: `tema::Tokens` con `claro() -> Tokens` y `oscuro() -> Tokens`; `main.rs` mantiene `Pagina { Inicio, Imprimir, Escanear, Herramientas }` y `Estado { modo_oscuro: bool, pagina: Pagina, .. }`.

- [ ] **Step 1: `cargo init --name printdesk` y añadir deps del stack**

`cargo add iced --features image,svg && cargo add exprim --features print && cargo add printpdf image rfd dirs chrono && cargo add --dev tempfile`

- [ ] **Step 2: Escribir test de tokens (falla)**

En `tema.rs`: `#[test] fn tema_oscuro_difiere()` — la superficie y el fondo oscuros no coinciden con los claros, y el primario claro es `#4C5BF5` y el oscuro `#6474FF` (valores del spec).

- [ ] **Step 3: `cargo test` → FAIL** (módulo vacío)

- [ ] **Step 4: Implementar `Tokens`** (constantes del spec) y en `main.rs` la ventana Iced con `iced::application(boot, update, view).title("PrintDesk")`, sidebar oscura con 4 botones que cambian `Estado::pagina`, contenido vacío por página y conmutador de tema. Nota 0.14: el título va con `.title()`, no como argumento.

- [ ] **Step 5: `cargo test` → PASS y `cargo build` compila**

- [ ] **Step 6: Commit** `feat: andamio printdesk con tema y navegación`

---

### Task 2: Núcleo — dispositivos

**Files:**
- Create: `src/nucleo/mod.rs`, `src/nucleo/dispositivos.rs`
- Test: `src/nucleo/dispositivos.rs` (unitarios inline)

**Interfaces:**
- Produces: `pub struct Impresora { pub nombre: String, pub por_defecto: bool, pub acepta: bool }`, `pub fn impresoras() -> Vec<Impresora>` (envuelve `exprim::print::spool::platform_spooler()`; error → vacío); `pub struct Escaner { pub id: String, pub descripcion: String }`, `pub fn escaners() -> Vec<Escaner>` (ejecuta `scanimage -L`), `pub fn parsear_scanimage_l(salida: &str) -> Vec<Escaner>`; `pub fn detectar() -> Dispositivos { impresoras, escaners }`.

- [ ] **Step 1: Tests con la salida real de esta máquina** (fixture, tomada el 2026-11-14):

```text
device `v4l:/dev/video0' is a Noname HP TrueVision HD Camera: HP Tru virtual device
device `epson2:libusb:003:012' is a Epson PID 1105 flatbed scanner
```

`parsear_scanimage_l` → un solo `Escaner { id: "epson2:libusb:003:012", .. }` (excluye ids que empiezan por `v4l:`). Otro test con cadena vacía → `[]`. Parseo estructural por `` ` `` y `' is a `` — nada de palabras localizadas.

- [ ] **Step 2: `cargo test` → FAIL**

- [ ] **Step 3: Implementar** (parse estructural; `scanimage -L` con `Command`, timeout 10 s vía hilo; falla → vacío)

- [ ] **Step 4: `cargo test` → PASS**

- [ ] **Step 5: Commit** `feat: detección de impresoras y escáneres`

---

### Task 3: Núcleo — impresión (opciones, rango, preview, envío)

**Files:**
- Create: `src/nucleo/impresion.rs`
- Test: `src/nucleo/impresion.rs`

**Interfaces:**
- Consumes: exprim `print::*`.
- Produces: `pub struct OpcionesImpresion { papel: Papel, orientacion: Orientacion, copias: u32, color: ColorImp, calidad: Calidad, margenes: Margenes, escala: Escala, rango: Rango }` con los enums `Papel{A4,Letter,Foto10x15}`, `Orientacion{Vertical,Horizontal}`, `ColorImp{Color,BlancoNegro}`, `Calidad{Borrador,Normal,Alta}`, `Margenes{Estrechos,Normales,Amplios}`, `Escala{Ajustar,Real,Porcentaje(f32)}`, `Rango{Todas,Paginas(Vec<(u32,u32)>)}`; `pub fn parsear_rango(s: &str, total: u32) -> Result<Rango, ErrorRango>`; `pub fn previsualizar(pdf: &[u8], opts: &OpcionesImpresion) -> Result<Vec<Vec<u8>>, ErrorImpresion>` (PNG por hoja, con márgenes del spooler si disponibles); `pub fn imprimir(spooler: &dyn Spooler, impresora: &str, doc: &[u8], titulo: &str, opts: &OpcionesImpresion) -> Result<String, ErrorImpresion>` (id del trabajo). `OpcionesImpresion::predeterminadas()` = A4, vertical, 1 copia, color, normal, normales, ajustar, todas.

- [ ] **Step 1: Tests de rango**: `"todas"→Todas`; `"2-5,8"→[(2,5),(8,8)]`; `"0"`, `"5-2"`, `"99"` con total 8, `"a"` → `Err`.

- [ ] **Step 2: Tests de preview con PDF de 3 páginas generado con printpdf** (helper en el test): predeterminadas → 3 hojas PNG (cabecera `\x89PNG`); `Rango::Paginas([(2,3)])` → 2 hojas; **PDF truncado** (`b"%PDF-roto"`) → `Err` sin pánico.

- [ ] **Step 3: Test de envío con `FileSpooler`** de exprim: `imprimir(..)` crea ficheros de hoja y devuelve id no vacío.

- [ ] **Step 4: `cargo test` → FAIL**

- [ ] **Step 5: Implementar** mapeando `OpcionesImpresion` → `exprim::print::PrintOptions` (A4→`PaperSize::A4`…; escala Porcentaje → `Scaling::Percent`; color/calidad → opciones `lp` cuando passthrough no aplique).

- [ ] **Step 6: `cargo test` → PASS**

- [ ] **Step 7: Commit** `feat: opciones de impresión, preview y envío`

---

### Task 4: Núcleo — PDFs de salida (escaneos e imágenes)

**Files:**
- Create: `src/nucleo/pdfsal.rs`
- Test: `src/nucleo/pdfsal.rs`

**Interfaces:**
- Consumes: `PaginaEscaneada` (Tarea 5 — define aquí el struct y Tarea 5 lo importa: `pub struct PaginaEscaneada { pub png: Vec<u8>, pub rotacion: u32 }`).
- Produces: `pub fn imagen_a_pdf(img: &[u8]) -> Result<Vec<u8>, ErrorPdf>` (1 página del tamaño de la imagen); `pub fn paginas_a_pdf(paginas: &[PaginaEscaneada], destino: &Path) -> Result<(), ErrorPdf>` (aplica rotación de 90°/180°/270° al componer).

- [ ] **Step 1: Tests**: `imagen_a_pdf` con un PNG 100×140 generado con `image` → PDF que reabierto con `exprim::RenderSession` da `page_count()==1` y `page_geometry(0)` con proporción 100:140; `paginas_a_pdf` con 3 páginas (la 2.ª con `rotacion: 90`) → PDF de 3 páginas que existe en `tempfile`; PNG vacío → `Err`.

- [ ] **Step 2: `cargo test` → FAIL**

- [ ] **Step 3: Implementar con printpdf** (embeber imagen sin re-codificar cuando el formato lo permita; rotación al añadir página).

- [ ] **Step 4: `cargo test` → PASS**

- [ ] **Step 5: Commit** `feat: generación de pdf de escaneos e imágenes`

---

### Task 5: Núcleo — escaneo

**Files:**
- Create: `src/nucleo/escaneo.rs`
- Test: `src/nucleo/escaneo.rs`

**Interfaces:**
- Consumes: `PaginaEscaneada` (Tarea 4).
- Produces: `pub struct OpcionesEscaneo { pub resolucion: u32, pub modo: Modo, pub formato_salida: FormatoSalida }` con `Modo{Color,Grises,Linea}`, `FormatoSalida{Pdf,Png,Jpg}` y `predeterminadas()` = 300/Color/Pdf; `pub fn args_scanimage(dispositivo: &str, o: &OpcionesEscaneo) -> Vec<String>`; `pub fn escanear(dispositivo: &str, o: &OpcionesEscaneo, cancel: Arc<AtomicBool>) -> Result<PaginaEscaneada, ErrorEscaneo>` (bloqueante; lo llama la UI en hilo). El binario es `std::env::var("PRINTDESK_SCANIMAGE").unwrap_or("scanimage")`.

- [ ] **Step 1: Tests**: `args_scanimage("epson2:x", 300/Color/Pdf)` == `["-d","epson2:x","--mode","Color","--resolution","300","--format","png"]`; con binario **falso** (script `printf 'PNG-falso'` marcado ejecutable en tempfile y `PRINTDESK_SCANIMAGE` apuntándolo) → `Ok(PaginaEscaneada{ png: b"PNG-falso".., rotacion:0 })`; script que duerme 30 s con `cancel=true` a los 200 ms → `Err(ErrorEscaneo::Cancelado)` en < 2 s.

- [ ] **Step 2: `cargo test` → FAIL**

- [ ] **Step 3: Implementar** (spawn, kill al cancelar, stderr como `ErrorEscaneo::Scanimage(msg)`).

- [ ] **Step 4: `cargo test` → PASS**

- [ ] **Step 5: Commit** `feat: escaneo por scanimage cancelable`

---

### Task 6: Núcleo — asistente DNI

**Files:**
- Create: `src/nucleo/dni.rs`
- Test: `src/nucleo/dni.rs`

**Interfaces:**
- Consumes: `PaginaEscaneada`.
- Produces: `pub enum PasoDni { EsperandoAnverso, EscaneandoAnverso, EsperandoReverso, EscaneandoReverso, Completo }`; `pub enum MensajeDni { Escanear, PaginaLista(PaginaEscaneada), Repetir, Cancelar }`; `pub struct AsistenteDni`; `AsistenteDni::nuevo()`, `.paso() -> PasoDni`, `.anverso() -> Option<&PaginaEscaneada>`, `.reverso() -> Option<&PaginaEscaneada>`, `.actualizar(MensajeDni)` (transiciones; `PaginaLista` en `Completo` devuelve a `EsperandoAnverso` para encadenar DNIs; `Repetir` descarta la última cara).

- [ ] **Step 1: Tests del flujo completo** (secuencia feliz: Escanear→PaginaLista→`EsperandoReverso`→Escanear→PaginaLista→`Completo` con ambas caras); `Repetir` en `EsperandoReverso` limpia anverso y vuelve a `EsperandoAnverso`; `Cancelar` desde cualquier paso → `EsperandoAnverso` vacío; `PaginaLista` inesperado se ignora.

- [ ] **Step 2: `cargo test` → FAIL → implementar → PASS → Commit** `feat: máquina de estados del asistente dni`

---

### Task 7: Núcleo — trabajos recientes

**Files:**
- Create: `src/nucleo/trabajos.rs`
- Test: `src/nucleo/trabajos.rs`

**Interfaces:**
- Produces: `pub struct Trabajo { pub nombre: String, pub detalle: String, pub tipo: TipoTrabajo, pub fecha_unix: i64 }`, `TipoTrabajo{Impresion,Escaneo}`; `pub fn registrar(dir: &Path, t: &Trabajo) -> Result<(), ErrorTrabajos>` y `pub fn recientes(dir: &Path, n: usize) -> Vec<Trabajo>` (JSON `trabajos.json`, máx. 20, más nuevo primero).

- [ ] **Step 1: Tests** con `tempfile`: registrar 3 → `recientes(10)` devuelve 3 en orden inverso; 25 registros → 20; fichero corrupto → `vec![]` sin pánico.

- [ ] **Step 2: FAIL → implementar → PASS → Commit** `feat: historial de trabajos recientes`

---

### Task 8: UI — Inicio

**Files:**
- Modify: `src/main.rs`, `src/ui/mod.rs`
- Create: `src/ui/inicio.rs`, `src/ui/widgets.rs`
- Test: visual (sin test automático; compila y abre)

**Interfaces:**
- Consumes: `nucleo::dispositivos::detectar()`, `nucleo::trabajos::*` (directorio de datos vía `dirs::data_dir()/printdesk`).

- [ ] **Step 1: Pantalla según spec**: tarjeta Epson EcoTank L375 con chips «Lista para imprimir»/«Escáner disponible» (o el error real), 3 accesos rápidos, recientes. Mensajes `Mensaje::Ir(Pagina)` desde los accesos.
- [ ] **Step 2: `cargo build` OK y comprobación visual manual (abrir 10 s)**
- [ ] **Step 3: Commit** `feat: pantalla de inicio con estado real`

---

### Task 9: UI — Imprimir

**Files:**
- Create: `src/ui/imprimir.rs`
- Modify: `src/main.rs`, `src/ui/widgets.rs`
- Test: compila + manual

**Interfaces:**
- Consumes: `nucleo::impresion::*`, `nucleo::pdfsal::imagen_a_pdf`, `nucleo::dispositivos::impresoras`, `rfd::FileDialog`.

- [ ] **Step 1: Carga**: botón «Abrir» con `rfd` (PDF/JPG/PNG); las imágenes pasan por `imagen_a_pdf` y siguen el mismo camino. shortcut: arrastrar y soltar queda para fase 2 si Iced no expone eventos de drop sin trabajo extra.
- [ ] **Step 2: Layout 3 paneles**: miniaturas (izq., ocultable con botón «»), visor central con zoom/ajuste (imagen PNG de la hoja activa), opciones (der., ocultable) con cada control del spec enlazado a `OpcionesImpresion`.
- [ ] **Step 3: Preview en hilo**: al cambiar opciones, `previsualizar` en `std::thread` + canal; el visor muestra «Calculando…» mientras.
- [ ] **Step 4: Imprimir**: botón → `imprimir(platform_spooler, ..)` en hilo; resultado como toast/estado con id; registra en `trabajos`.
- [ ] **Step 5: `cargo build` + manual (PDF real de 3 págs) → Commit** `feat: pantalla imprimir con preview fiel`

---

### Task 10: UI — Escanear

**Files:**
- Create: `src/ui/escanear.rs`
- Modify: `src/main.rs`, `src/ui/widgets.rs`
- Test: compila + manual

**Interfaces:**
- Consumes: `nucleo::escaneo::*`, `nucleo::pdfsal::paginas_a_pdf`.

- [ ] **Step 1: Controles** (resolución/color/salida como en el spec) y botón «Escanear página» → hilo con `escanear` y spinner + cancelar.
- [ ] **Step 2: Tira de páginas**: miniaturas con girar (rotacion += 90), eliminar y reordenar arrastrando (mouse_area por tarjeta: pulsar→umbral→soltar sobre otra intercambia; botones ←/→ como respaldo siempre visibles).
- [ ] **Step 3: Guardar**: `paginas_a_pdf` → `~/Escaneos/AAAA-MM-DD_HHMM.pdf` (carpeta creada si no existe), botón «Abrir carpeta» (`xdg-open`), registra en `trabajos`.
- [ ] **Step 4: `cargo build` + manual → Commit** `feat: pantalla escanear con tira editable`

---

### Task 11: UI — Herramientas y asistente DNI

**Files:**
- Create: `src/ui/herramientas.rs`
- Modify: `src/main.rs`
- Test: compila + manual

**Interfaces:**
- Consumes: `nucleo::dni::*`, `nucleo::escaneo::*`, `nucleo::pdfsal::paginas_a_pdf`.

- [ ] **Step 1: Página Herramientas**: tarjeta activa «DNI por ambas caras» + 3 tarjetas «fase 2» apagadas (recorte, Word/Excel, dúplex manual), según la maqueta aprobada.
- [ ] **Step 2: Asistente**: tres pasos visuales ligados a `AsistenteDni`, escaneo en hilo, «Repetir anverso», y al completar guarda `~/Escaneos/DNI_AAAA-MM-DD.pdf` y ofrece empezar otro.
- [ ] **Step 3: `cargo build` + manual → Commit** `feat: herramientas y asistente dni`

---

### Task 12: Entrega — instalador, icono y README

**Files:**
- Create: `instalar.sh`, `assets/printdesk.svg`, `README.md` (reemplaza el actual de una línea)
- Test: `tests/instalador.bash` (simple script de comprobación)

**Interfaces:**
- Consumes: binario `target/release/printdesk`.

- [ ] **Step 1: `instalar.sh`**: `cargo build --release` → copia a `~/.local/bin/printdesk`, instala icono y `~/.local/share/applications/printdesk.desktop` (sin root; idempotente).
- [ ] **Step 2: Test bash con `HOME` en tempfile**: ejecuta el instalador con binario ya compilado (`SKIP_BUILD=1`) y comprueba que existen `bin/printdesk` y el `.desktop`.
- [ ] **Step 3: Ejecutar test → PASS → Commit** `feat: instalador de usuario y readme`

---

### Task 13: Aceptación real con la Epson L375

**Files:**
- Create: `docs/acta-aceptacion.md` (checklist marcada)

- [ ] **Step 1: Con Luis y su L375**: imprimir un PDF de varias páginas (A4, rango 2-3, 2 copias); imprimir una foto JPG; escanear 3 hojas seguidas y guardar un PDF; hacer un DNI por ambas caras; cancelar un escaneo a mitad; desconectar el USB y ver el estado en Inicio; instalar con `instalar.sh` y abrir desde el menú.
- [ ] **Step 2: Marcar el acta, corregir lo que falle (en rama) y Commit** `docs: acta de aceptación v1`

---

## Notas de auto-revisión

- Cobertura del spec: Inicio (T8), Imprimir (T9), Escanear (T10), Herramientas/DNI (T6+T11), identidad/tema (T1), instalador (T12), estados reales sin inventar (T2+T8), fases 2 fuera (mejoras-futuras.md). Sin huecos.
- Los pasos de UI sin test automático se verifican compilando + uso manual; la lógica que puede romperse vive toda en `nucleo` con tests (Review Focus anclado).
- Tipos compartidos (`PaginaEscaneada`, `OpcionesImpresion`) definidos una vez y consumidos por nombre en tareas vecinas.

## Estado de ejecución

- Implementadas tareas 1–12; registro técnico en el ledger del plan mientras dura la ejecución.
- Tarea 13 pendiente: pruebas físicas con Luis y Epson L375 preparada. No se consume papel ni se capturan documentos personales en las pruebas automáticas.
- Stack efectivo: Iced 0.14, exprim 0.4, printpdf 0.12.9, image, rfd, dirs, chrono, tempfile; serde/serde_json para historial; iced_test solo para desarrollo. Inter y su licencia están en `assets/`.
- Diferencias justificadas en la sección «Precisiones verificadas durante la construcción» del diseño: CUPS con vocabulario del PPD, foto 4×6, sesiones de DPI fijo y limitación de arrastre de archivos en Wayland.

# Diseño: PrintDesk — estudio de impresión y escaneo para la Epson L375

Fecha: 2026-11-14 (v2, tras revisión de Luis) · Estado: pendiente de aprobación ·
Rama: `feat/app-impresora-escaner` (main intacto)

## Intención y por qué

Luis maneja su **Epson EcoTank L375** a puro comando hoy. Quiere **PrintDesk**: una
aplicación de escritorio **nativa, moderna y con identidad visual propia** en Rust,
que concentre imprimir y escanear con vista previa real, y que crezca con
herramientas (la primera: asistente de DNI por ambas caras) sin rehacer la app.

Éxito = abrir PrintDesk desde el menú de aplicaciones y no volver a la terminal.

## Metas (v1)

1. **Inicio**: tarjeta de la Epson EcoTank L375 con estado real (lista para
   imprimir / escanear / apagada o sin conexión), accesos rápidos y trabajos
   recientes (imprimir y escanear).
2. **Imprimir**: cargar PDF o imagen (JPG/PNG) y que **la vista previa sea la
   protagonista**: miniaturas a la izquierda (panel ocultable), visor central
   grande con zoom, opciones a la derecha (panel ocultable): papel, orientación,
   copias, color, calidad, márgenes, escala, rango de páginas. La vista previa
   representa el trabajo tal como saldrá, antes de enviarlo.
3. **Escanear**: escanear varias hojas seguidas (placa A4 completa, sin recorte
   en v1), verlas, **reordenarlas, girarlas y eliminarlas**, y guardarlas como
   **un único PDF**. Resolución, color y formato de salida configurables.
4. **Herramientas**: apartado presente desde v1 con el **asistente DNI por ambas
   caras** (escanea anverso → «gira el DNI» → escanea reverso → PDF listo).
   La navegación y la arquitectura permiten añadir herramientas nuevas sin
   rehacer la aplicación.
5. Detección automática de dispositivos (CUPS + SANE) con **estados reales**:
   nada de niveles de tinta ni funciones que el controlador no ofrezca.
6. Identidad visual propia (ver abajo), modo claro y oscuro, ventanas adaptables,
   interfaz en español. Instalador a nivel de usuario.

## No-metas (v1) — todas pospuestas a fases siguientes (`docs/mejoras-futuras.md`)

- Recorte inteligente del escaneo (DNI a tamaño de tarjeta).
- Imprimir Word/Excel (llegará vía conversión LibreOffice).
- Dúplex manual guiado al imprimir (la L375 no imprime dos caras sola).
- OCR, ADF, niveles de tinta, gestión de colas.
- Multiplataforma (esto es para este Linux; nada impide portarlo).

## Anti-metas

- Sustituir a CUPS/SANE o tocar su configuración: la app los usa, no los arregla.
- Inventar estados del dispositivo que el controlador no informe.
- Pinta de «utilidad antigua de Linux»: si un panel parece de sysadmin, está mal.

## Restricciones

- Fedora + CUPS + SANE ya funcionando (`Epson_L375` en `lpstat -e`;
  `epson2` en `scanimage -L`). Rust 1.100 instalado.
- Impresión y escaneo por las herramientas de línea de comandos (`lp`, `lpstat`,
  `scanimage`) — sin FFI. Validado: exprim 0.4 detecta la impresora con CUPS en
  español y genera vistas previas (prueba real 2026-11-14).
- Todo en la rama de desarrollo; `main` no se toca.
- Skills del agente a nivel proyecto, nunca global (regla de Luis).

## Tecnología de interfaz: Iced (decisión v2)

Luis pidió evaluar **Iced vs egui** buscando una app moderna y personalizada, no
una herramienta austera. Comparativa (noviembre de 2026):

| Criterio | Iced 0.14 | egui 0.36 |
|---|---|---|
| Mantenimiento | Activo (0.14 dic-2025; hito 0.14 cerrado sep-2026, desarrollo diario) | Muy activo, pero rompe API cada versión menor y arrastra al ecosistema |
| Apariencia personalizable | **Fuerte**: `Theme`/`Palette` por widget, esquinas redondeadas, reacción al tema claro/oscuro del sistema | Limitada a `Style`; una interfaz ricamente estilizada es cara (lo reconocen sus usuarios avanzados) |
| Paneles ocultables | **`pane_grid`**: divisiones, redimensionado con ratón, ocultar/mostrar | Manual: calcular rectángulos a mano cada frame |
| Previsualización de documentos | **`image::Viewer`** con zoom/pan incluido; decodificación concurrente de imágenes | Factible pero manual (pintar texturas a mano) |
| Rendimiento escritorio | Retenido: solo redibuja al cambiar (menos CPU en reposo); algo más de RAM (~143 MB) | Inmediato: redibuja mucho; más ligero de RAM (~72 MB) |
| Linux | wgpu (Vulkan/GL), flags `x11`/`wayland` | OpenGL o wgpu |
| Tests de UI | **Modo headless + pruebas end-to-end oficiales** | Community-driven (`egui_kittest`) |

**Decisión: Iced.** El coste (algo más de RAM y binario) es irrelevante en
escritorio, y estilo propio, paneles, visor con zoom y tests de UI son justamente
lo que este proyecto necesita. Viabilidad verificada compilando una ventana de
prueba en esta máquina (ver «Validaciones»). egui queda documentado como
alternativa si Iced tropezara con algo insalvable; el núcleo (sin UI) queda
separado de la interfaz para que cambiar de toolkit no reescriba la lógica.

### Stack completo

- **Iced 0.14** — interfaz (Elm: estado + mensajes + vista).
- **exprim 0.4** (`print`) — abrir/render PDF, imposición (papel, márgenes,
  escala, rangos), envío a CUPS, `FileSpooler` para tests.
- **scanimage** (subproceso) — escaneo; modos y DPI del dispositivo.
- **printpdf** — escribir el PDF único de los escaneos y el envoltorio de
  imágenes para imprimir por la misma tubería.
- **image** — decodificar imágenes para la UI.
- Hilos de fondo (`std::thread` + canal, o `Task` de Iced) para escanear,
  imprimir y generar vistas previas: **la interfaz nunca se congela**.

## Arquitectura

```
src/
  main.rs        – arranque, tema (claro/oscuro), navegación por páginas
  ui/
    tema.rs      – tokens visuales (paleta índigo, radios, tipografía) y
                   Theme de Iced claro/oscuro; todo widget estiliza desde aquí
    inicio.rs    – pantalla de inicio
    imprimir.rs  – pantalla imprimir (miniaturas | visor | opciones)
    escanear.rs  – pantalla escanear
    herramientas.rs – lista de herramientas y enrutado a cada asistente
    widgets.rs   – piezas propias (tarjeta de página, botón principal, toggles)
  nucleo/        – SIN dependencia de Iced: toda la lógica y los tests
    dispositivos.rs – impresoras (lpstat/exprim), escáner (scanimage -L),
                      estados reales y caché
    impresion.rs – opciones de trabajo → exprim (imposición + envío)
    escaneo.rs   – subproceso scanimage → páginas en memoria
    pdfsal.rs    – páginas escaneadas → PDF único; imagen → PDF de 1 página
    trabajos.rs  – historial reciente (JSON en ~/.local/share/printdesk/)
    dni.rs       – máquina de estados del asistente DNI (anverso→reverso→PDF)
  iconos/        – SVG de línea propios, trazo consistente
instalar.sh      – binario → ~/.local/bin + .desktop con icono (sin root)
```

- **Navegación**: enum `Pagina { Inicio, Imprimir, Escanear, Herramienta(Id) }`
  en la barra lateral. Añadir una herramienta = nuevo módulo + una entrada:
  sin rehacer nada (pedía Luis en el punto 6).
- **Núcleo separado de la UI**: la interfaz solo dibuja y manda mensajes; así lo
  exige también la lección de la comparativa (cambiar toolkit no reescribe
  lógica) y permite tests del núcleo sin abrir ventana.

## Identidad visual (obligatoria)

Tokens definidos una vez en `ui/tema.rs` y usados por todo widget:

| Token | Claro | Oscuro |
|---|---|---|
| Fondo principal | `#F5F6FA` gris suave | `#15171F` |
| Superficie (tarjetas) | `#FFFFFF` | `#1E212B` |
| Barra lateral | `#1B1E2B` (oscura en ambos modos) | `#101221` |
| Primario índigo | `#4C5BF5` | `#6474FF` |
| Primario al pasar | `#3D49D6` | `#7B87FF` |
| Texto | `#1E2230` | `#E9EBF5` |
| Texto secundario | `#697086` | `#9AA1B9` |
| Borde | `#E4E7F0` | `#2A2E3F` |
| Correcto / aviso | `#27A06A` / `#D98A2B` | ídem adaptados |

- Radios: 12 px tarjetas, 10 px botones, 999 px píldoras.
- Tipografía: **Inter** (incluida en el binario;.system-ui de respaldo).
- Iconos: SVG de línea, trazo 1.75, esquinas ligeramente redondeadas — el mismo
  set en toda la app (impresora, escáner, casa, herramientas, luna/sol).
- La barra lateral es oscura en ambos modos (identidad), el contenido cambia.
- Modo claro/oscuro: conmutador en la cabecera + seguir al sistema (Iced 0.14
  reacciona al tema del sistema).
- Adaptable: los paneles laterales se ocultan con un botón (y con el teclado);
  a ventana estrecha, opciones pasan a un panel deslizante.

## Pantallas (alcance exacto v1)

### Inicio
- Tarjeta grande «Epson EcoTank L375» con dos luces de estado reales:
  «Lista para imprimir» (cola CUPS acepta trabajos) y «Escáner disponible»
  (SANE ve el dispositivo). Si algo falla, el estado y el motivo, sin inventar.
- Accesos rápidos: Imprimir documento · Escanear · Asistente DNI.
- Recientes: últimos trabajos (nombre, páginas, fecha, tipo imprimir/escanear),
  con reabrir.

### Imprimir
- Cargar: botón «Abrir» + arrastrar y soltar (PDF, JPG, PNG).
- Izquierda (ocultable): miniaturas de páginas/hojas; clic = salta el visor.
- Centro: **visor grande** con zoom (rueda y botones), ajuste a ancho, y la hoja
  tal como se imprimirá (papel + márgenes + escala), renderizada con exprim.
- Derecha (ocultable): impresora (preseleccionada la L375), papel (A4/Letter/
  Legal/10×15…, por defecto **A4**), orientación, copias, color (color/b/n),
  calidad (borrador/normal/alta), márgenes (estrechos/normales/amplios),
  escala (ajustar / real / %), rango de páginas (p. ej. 2-5, 8).
- Botón «Imprimir»: envía con exprim (pass-through si nada cambia geometría);
  confirmación con número de trabajo y error claro si CUPS rechaza.

### Escanear
- Controles: resolución (150/300/600), color (color/escala de grises/línea),
  formato de salida (PDF único, o PNG/JPG por página).
- «Escanear página»: escanea (hilo aparte, progreso y cancelar) y añade la
  página a la tira; escaneos consecutivos para varias hojas.
- Tira de páginas: miniaturas con girar 90°, eliminar y **reordenar arrastrando**.
- «Guardar»: PDF único en `~/Escaneos/AAAA-MM-DD_HHMM.pdf` (o imágenes), botón
  «Abrir carpeta». A4 completo, sin recorte (decisión de Luis).

### Herramientas → Asistente DNI
- Tarjeta por herramienta; v1 solo «DNI por ambas caras».
- Pasos guiados: 1) «Coloca el DNI y pulsa Escanear» (anverso) 2) «Gira el DNI
  y pulsa Escanear» (reverso) con miniatura de lo capturado 3) «Guardar PDF»
  (`~/Escaneos/DNI_AAAA-MM-DD.pdf`). Repetir lado si sale mal, cancelar, y
  hacer varios DNIs seguidos.

## Pruebas

- Núcleo (sin UI): opciones→imposición con `FileSpooler`; máquina de estados
  DNI; reordenar/girar/eliminar páginas; parseo de `lpstat`/`scanimage -L` con
  salidas reales (la máquina está en español); PDF generado se reabre con exprim.
- UI: modo headless de Iced para las pantallas principales (abrir, cambiar
  opciones, flujo DNI).
- Aceptación real con la L375 al final (imprimir un PDF, DNI por ambas caras).

## Validaciones hechas en esta máquina (2026-11-14)

- exprim 0.4 compila con Rust 1.100, abre y renderiza un PDF real, detecta la
  `Epson_L375` (CUPS en español) y compone vista previa. Nada enviado a imprimir.
- Iced 0.14: compila con Rust 1.100 (wgpu 27) y **abre ventana real** en el
  escritorio de esta máquina (prueba de 6 s, 2026-11-14).

## Entrega

- Binario + `instalar.sh` (usuario): copia a `~/.local/bin/printdesk` y crea
  `~/.local/share/applications/printdesk.desktop` con icono. Sin root.

## Queda a criterio del constructor

- Detalles finos de composición (espaciados exactos, orden de controles).
- Cómo mostrar el progreso del escaneo en curso (barra + cancelar).
- Límite de páginas en memoria por sesión de escaneo.

## Notas de proceso

- Skills: brainstorming + frontend-design + ponytail + TDD. Ninguna skill extra
  de Rust necesaria por ahora; si hiciera falta, instalación a nivel proyecto.
- Fase 2 planificada en `docs/mejoras-futuras.md` (recorte DNI, Word/Excel,
  dúplex manual guiado…).
- Este documento (v2) sustituye al v1; las decisiones de Luis quedan marcadas.

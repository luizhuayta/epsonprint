# Mejoras futuras de EpsonPrint

Lo que **no** entra en la v1, a propósito. Aquí queda apuntado para decidir más adelante.
El diseño de la v1 no cierra la puerta a ninguna de estas.

| Mejora | Qué aporta | Qué haría falta | Tamaño orientativo |
|---|---|---|---|
| **Dúplex manual guiado (imprimir)** | Imprimir a dos caras: la app imprime las hojas impares, te dice «voltea el papel» y sigue con las pares en orden inverso. | Partir el trabajo en dos (impares/pares), UI de pausa entre tandas, decidir borde de volteo. Ojo con copias múltiples. | Media — parecida al flujo dos caras de escaneo |
| **Imprimir Excel/Word** | Cargar `.xlsx`/`.docx` directamente y que se impriman. | Convertir a PDF por dentro con LibreOffice (`soffice --headless --convert-to pdf`) ya instalado; la app ya imprime PDF. | Pequeña |
| **Recorte automático del escaneo** | Que el DNI ocupe su tamaño real en el PDF, no una hoja A4 casi blanca. | Detección de bordes blancos al guardar (recortar márgenes vacíos). | Pequeña-media |
| **Fuente ADF** (si el driver la expone) | Cargar varias hojas del alimentador de golpe. | Comprobar si `scanimage` expone fuente ADF en la L375; si no, no es posible. | Pequeña (si el driver lo permite) |
| **OCR de los escaneos** | Buscar texto dentro de los PDF escaneados. | Motor OCR (p. ej. Tesseract) integrado como subproceso. | Media |
| **Niveles de tinta** | Ver cuánta tinta queda sin abrir la web de la impresora. | Consultar el estado IPP de la impresora (la L375 lo responde por la red). | Pequeña-media |
| **Cola de impresión visible** | Ver/cancelar trabajos enviados desde la app. | Leer `lpstat -o` y `cancel`; ya existe en exprim. | Pequeña |
| **Vista previa con zoom y páginas múltiples** | Repasar un PDF largo cómodamente antes de imprimir. | Paginación de miniaturas y zoom en la pestaña Imprimir. | Pequeña |
| **Tema/estilo propio** | Que la app tenga un look más cuidado. | Colores y tipografía de egui. | Pequeña |

Sugerencia de orden cuando toque: **Excel/Word** y **recorte automático** son las que
más se notan con menos esfuerzo; el dúplex manual guiado es la gran pieza siguiente.

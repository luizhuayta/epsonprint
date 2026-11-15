# PrintDesk — mejoras posteriores

## Segunda fase acordada

- **Recorte inteligente**: detectar o seleccionar el documento dentro del A4. Incluye composición de las dos caras del DNI sobre una hoja si se necesita.
- **Word y Excel**: convertir con LibreOffice a PDF en un directorio temporal privado, sin alterar el archivo original. Comprobar fidelidad de tablas/fuentes.
- **Dúplex manual guiado**: tandas impares/pares, pausa para voltear, orden de salida y copias múltiples. La L375 no tiene dúplex automático.

## Otras mejoras posibles, no comprometidas

- OCR con Tesseract para PDF con texto buscable.
- Cola de impresión con seguimiento y cancelación de trabajos. «CUPS aceptó» no significa «se imprimió físicamente».
- Detección de dispositivos periódica y exposición de los motivos detallados del backend.
- Mejoras de accesibilidad y atajos del visor; persistir preferencias y sesiones al cerrar.
- Mezclar distintas resoluciones en una misma sesión sin cambiar tamaños físicos.
- Mejor rendimiento en documentos de cientos de páginas: vista previa incremental y caché limitada.
- Arrastrar archivos en Wayland cuando lo admita el backend de ventanas.
- Portar a otros sistemas sin confundir CUPS/SANE con controladores de Windows.

ADF y niveles de tinta solo deben añadirse si el **hardware y el controlador** los anuncian. No asumir que la L375 dispone de ellos.

# PrintDesk — aceptación con Epson EcoTank L375

Estado: **pendiente de prueba física con Luis**. Las pruebas automatizadas no sustituyen papel, tinta ni la orientación real del cristal.

## Verificado sin consumir papel ni escanear documentos personales

- [x] Pruebas unitarias y de integración con CUPS/SANE falsos.
- [x] PDF multipágina generado con printpdf y reabierto con exprim.
- [x] Cancelación de subproceso y lectura de una salida mayor que un pipe.
- [x] Guardado sin sobrescribir documentos anteriores.
- [x] Navegación Iced con clics simulados y capturas headless claro/oscuro.
- [x] Instalador probado en HOME temporal, con espacios e idempotencia.
- [x] Opciones del PPD L375 consultadas sin modificar la cola.

## Con Luis y la impresora preparada

- [ ] Abrir desde el menú y comprobar detección de Epson_L375 y escáner epson2.
- [ ] PDF A4 de varias páginas: rango 2–3, 2 copias, color y gris; confirmar orden y márgenes sobre papel.
- [ ] Cambiar escala a 80 % y orientación; comparar preview y hoja impresa.
- [ ] Normal/Alta: confirmar que el PPD recibe la selección y el resultado físico cambia.
- [ ] Imprimir JPEG en A4 y en foto 4×6 con papel adecuado.
- [ ] Escanear tres hojas a 150/300/600 dpi (sesiones separadas); girar, reordenar, quitar y guardar.
- [ ] Cancelar una captura real y volver a escanear.
- [ ] DNI: anverso, giro manual, reverso, repetir lado, guardar las dos páginas y abrir el PDF.
- [ ] Desconectar USB y pulsar Actualizar dispositivos; comprobar mensajes, sin inventar disponibilidad física.
- [ ] Cerrar con páginas pendientes y comprobar el aviso de pérdida de sesión.

Anotar aquí los resultados y cualquier ajuste de calibración necesario antes de considerar la versión aceptada.

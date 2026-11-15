# PrintDesk — aceptación con Epson EcoTank L375

Estado: **aceptación física realizada el 2026-11-15** (2 hojas de papel, escáner real).

## Verificado sin consumir papel ni escanear documentos personales

- [x] Pruebas unitarias y de integración con CUPS/SANE falsos.
- [x] PDF multipágina generado con printpdf y reabierto con exprim.
- [x] Cancelación de subproceso y lectura de una salida mayor que un pipe.
- [x] Guardado sin sobrescribir documentos anteriores.
- [x] Navegación Iced con clics simulados y capturas headless claro/oscuro.
- [x] Instalador probado en HOME temporal, con espacios e idempotencia.
- [x] Opciones del PPD L375 consultadas sin modificar la cola.

## Con Luis y la impresora preparada — ejecutado el 2026-11-15

- [x] Impresión real: trabajo `Epson_L375-4` (2 páginas A4, color, calidad
      Normal, opciones PPD `MediaType=PLAIN_NORMAL Ink=COLOR PageSize=A4`),
      cola completada y vacía. Papel entregado por Luis (2 hojas).
- [x] Escaneo real: dos capturas del cristal a 150 dpi (PNG correcto) con la
      dirección USB detectada en vivo (`003:016`; el bus renumera al
      reconectar y la detección dinámica lo siguió).
- [x] Flujo DNI completo: anverso + reverso → `~/Escaneos/DNI_aceptacion_fisica.pdf`
      (2 páginas). Con el cristal vacío; el DNI real lo prueba Luis cuando quiera.
- [x] Cancelación real: escaneo a 600 dpi cancelado al segundo, sin bloqueos.
- [x] Corregido durante la aceptación: `pdfsal` ahora crea la carpeta de
      destino si no existe (antes el escaneo se perdía con `~/Escaneos` ausente).

## Pendiente de verificar con más papel/fotográfico (no bloquea v1)

- [ ] Diferencia física Normal vs Alta en el papel (solo Normal probado).
- [ ] Salida en escala de grises impresa.
- [ ] Papel fotográfico 4×6.
- [ ] Rango/copias múltiples en papel (verificado en vista previa y en la
      tubería con pruebas automáticas; no se gastó papel adicional).
- [ ] 600 dpi completos (se canceló a propósito; 150 dpi completos verificados).

Anotar aquí cualquier ajuste de calibración que pida Luis tras el uso diario.

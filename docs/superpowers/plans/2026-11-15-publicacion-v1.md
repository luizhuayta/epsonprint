# Publicación de PrintDesk 0.1.0

Objetivo: que un visitante de GitHub entienda qué hace PrintDesk, sus límites y cómo instalarlo. Sin añadir funciones a la app ni empaquetados AppImage/Flatpak. Luis pidió publicar y continuar; autorizó MIT en el formulario de licencia.

## Entrega mínima

- [x] Corregir los avisos de Clippy sin alterar el comportamiento; formato, Clippy estricto y 48 pruebas normales pasan. Build release se verifica antes de integrar.
- [x] README para usuarios con capturas nativas, instalación desde código, uso, desinstalación y límites conocidos; licencia MIT, changelog 0.1.0 y guía breve de contribución.
- [ ] CI de GitHub en Ubuntu: formato, Clippy, tests normales, build release y prueba aislada del instalador. Nunca ejecutar pruebas físicas ignoradas.
- [ ] Verificar los cambios, integrarlos sin descartar trabajo ajeno y subirlos. Comprobar el resultado real de CI.
- [ ] Crear el tag/release v0.1.0 solo cuando el commit publicado pase CI; sin binarios que prometan compatibilidad no probada. Verificar en GitHub la release y los enlaces.

## Restricciones

No ejecutar otra tanda física ni capturar documentos personales. `.atl/` es trabajo ajeno y no se incluye. Inter conserva su licencia SIL OFL. La revisión independiente no está disponible por cuota: declarar esa limitación, no darla por aprobada.

#!/usr/bin/env bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")/.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/printdesk-install.XXXXXX")
trap 'rm -rf -- "$tmp"' EXIT
mkdir -p "$tmp/proyecto/assets" "$tmp/proyecto/target/release" "$tmp/casa con espacios"
cp "$repo/instalar.sh" "$tmp/proyecto/"
cp "$repo/assets/printdesk.svg" "$tmp/proyecto/assets/"
printf '#!/bin/sh\nexit 0\n' > "$tmp/proyecto/target/release/printdesk"
for vuelta in 1 2; do
    HOME="$tmp/casa con espacios" XDG_DATA_HOME="$tmp/datos" SKIP_BUILD=1 bash "$tmp/proyecto/instalar.sh"
done
test -x "$tmp/casa con espacios/.local/bin/printdesk"
test -s "$tmp/datos/icons/hicolor/scalable/apps/printdesk.svg"
desktop="$tmp/datos/applications/printdesk.desktop"
test -s "$desktop"
grep -Fq 'Name=PrintDesk' "$desktop"
grep -Fq "Exec=\"$tmp/casa con espacios/.local/bin/printdesk\"" "$desktop"
if command -v desktop-file-validate >/dev/null; then desktop-file-validate "$desktop"; fi
printf 'Instalador: rutas con espacios, XDG e idempotencia correctos.\n'

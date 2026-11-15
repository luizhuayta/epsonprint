#!/usr/bin/env bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")" && pwd)
if [[ ${SKIP_BUILD:-0} != 1 ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
    cargo build --release --locked --manifest-path "$repo/Cargo.toml"
fi
bin="$HOME/.local/bin/printdesk"
datos=${XDG_DATA_HOME:-$HOME/.local/share}
install -d "$HOME/.local/bin" "$datos/applications" "$datos/icons/hicolor/scalable/apps"
install -m 755 "$repo/target/release/printdesk" "$bin"
install -m 644 "$repo/assets/printdesk.svg" "$datos/icons/hicolor/scalable/apps/printdesk.svg"
exec_path=$(printf '%s' "$bin" | sed 's/[\\"`$]/\\&/g')
cat > "$datos/applications/printdesk.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=PrintDesk
Comment=Imprimir documentos y escanear con CUPS y SANE
Exec="$exec_path"
Icon=printdesk
Terminal=false
Categories=Office;
StartupNotify=true
EOF
if command -v update-desktop-database >/dev/null; then update-desktop-database "$datos/applications"; fi
printf 'PrintDesk instalado para tu usuario. Ábrelo desde el menú de aplicaciones.\n'

#!/usr/bin/env bash
# Uso: ./perfiles.sh ruta | instalar | activar <perfil> | listar
set -eu
[ $# -gt 0 ] || { sed -n 2p "$0"; exit 1; }
AQUI=$(cd "$(dirname "$0")" && pwd)
PERFILES=$(cd "$AQUI/../perfiles" && pwd)   # carpeta con los mw3_*.cfg
[ "$1" = listar ] && { ls "$PERFILES" | sed -n 's/^mw3_\(.*\)\.cfg$/\1/p'; exit 0; }
# Ruta de storage/iw5: PLUTONIUM_STORAGE si la defines, si no se busca en prefijos conocidos
# (Bottles flatpak/nativo, Bodega, Lutris, Wine, Proton). Si hay 0 o varios, define la variable.
if [ -z "${PLUTONIUM_STORAGE:-}" ]; then
  ENCONTRADAS=$(find "$HOME/.var/app/com.usebottles.bottles/data/bottles" "$HOME/.local/share/bottles" \
    "$HOME/.local/share/bodega" "$HOME/.wine" "$HOME/Games" "$HOME/.local/share/lutris" \
    "$HOME/.local/share/Steam/steamapps/compatdata" -maxdepth 12 -type d -path '*/Plutonium/storage/iw5' 2>/dev/null || true)
  case "$(printf '%s' "$ENCONTRADAS" | grep -c .)" in
    1) PLUTONIUM_STORAGE=$ENCONTRADAS ;;
    0) echo "No encontre Plutonium. Define PLUTONIUM_STORAGE=<...>/Plutonium/storage/iw5"; exit 1 ;;
    *) echo "Hay varias, define PLUTONIUM_STORAGE con una de estas:"; echo "$ENCONTRADAS"; exit 1 ;;
  esac
fi
STORAGE=$PLUTONIUM_STORAGE
[ -d "$STORAGE" ] || { echo "No existe $STORAGE"; exit 1; }
case "${1:-}" in
  instalar)
    mkdir -p "$STORAGE/main"
    cp -v "$PERFILES"/mw3_*.cfg "$STORAGE/main/"
    echo "Listo. En LanLauncher pulsa 'Refresh config list'." ;;
  activar)
    f="$PERFILES/mw3_${2:?falta perfil, ej: tdm_bots}.cfg"
    [ -f "$f" ] || { echo "No existe $f"; exit 1; }
    [ -f "$STORAGE/server.cfg" ] && cp "$STORAGE/server.cfg" "$STORAGE/server.cfg.bak-$(date +%Y%m%d-%H%M%S)"
    cp -v "$f" "$STORAGE/server.cfg"
    echo "Activo para el proximo arranque del servidor (no afecta uno ya encendido)." ;;
  ruta) echo "$STORAGE" ;;
  listar) ls "$PERFILES" | sed -n 's/^mw3_\(.*\)\.cfg$/\1/p' ;;
  *) sed -n '2p' "$0"; exit 1 ;;
esac

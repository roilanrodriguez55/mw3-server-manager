#!/usr/bin/env bash
# Uso: ./servidor.sh <perfil> [puerto]   ej: ./servidor.sh chico_ffa_nobots 5000
# Activa el perfil (lo copia sobre server.cfg) y lanza el servidor LAN con Wine, sin LanLauncher.
# Variables opcionales: PLUTONIUM_STORAGE (ver perfiles.sh), WINE (binario de wine), MW3_DIR (carpeta de MW3, ruta de Linux).
set -eu
AQUI=$(cd "$(dirname "$0")" && pwd)
PERFIL=${1:?falta el perfil. Disponibles: $("$AQUI/perfiles.sh" listar | tr '\n' ' ')}
PUERTO=${2:-5000}
STORAGE=$("$AQUI/perfiles.sh" ruta)
PLU=${STORAGE%/storage/iw5}                      # .../Plutonium
USUARIO=$(printf '%s' "$STORAGE" | sed -n 's#.*/drive_c/users/\([^/]*\)/.*#\1#p')
PREFIJO=${STORAGE%%/drive_c/*}                   # prefijo de Wine (el bottle)
[ -d "$PREFIJO/drive_c" ] || PREFIJO=${PREFIJO%/*}

# Wine: el del bottle si lo encuentra (Bottles usa GE-Proton aqui), si no el del sistema
if [ -z "${WINE:-}" ]; then
  for w in "$HOME"/.local/share/Steam/compatibilitytools.d/GE-Proton*/files/bin/wine \
           "$HOME"/.local/share/bodega/runtimes/wine-*/bin/wine "$(command -v wine || true)"; do
    [ -x "$w" ] && WINE=$w && break
  done
fi
[ -x "${WINE:-}" ] || { echo "No encontre wine. Define WINE=/ruta/a/wine"; exit 1; }

# Carpeta de MW3: la que Plutonium ya tiene configurada
if [ -z "${MW3_DIR:-}" ]; then
  MW3_WIN=$(sed -n 's/.*"iw5Path":"\([^"]*\)".*/\1/p' "$PLU/config.json" | sed 's/\\\\/\\/g')
  [ -n "$MW3_WIN" ] || { echo "No pude leer iw5Path de $PLU/config.json. Define MW3_DIR."; exit 1; }
else
  MW3_WIN="Z:$(printf '%s' "$MW3_DIR" | tr / '\\')"
fi

ss -uln | grep -q ":$PUERTO " && { echo "El puerto UDP $PUERTO ya esta en uso (otro servidor abierto?). Cierralo o usa otro puerto."; exit 1; }
"$AQUI/perfiles.sh" activar "$PERFIL"

LOG="$AQUI/servidor-$PERFIL.log"
echo "Wine: $WINE"; echo "Prefijo: $PREFIJO"; echo "MW3: $MW3_WIN"; echo "Log: $LOG"
# el bootstrapper busca games\iw5mp.exe relativo a la carpeta actual: hay que estar en la de Plutonium
cd "$PLU"
WINEPREFIX=$PREFIJO WINEDEBUG=-all setsid nohup "$WINE" \
  "C:\\users\\$USUARIO\\AppData\\Local\\Plutonium\\bin\\plutonium-bootstrapper-win32.exe" \
  iw5mp "$MW3_WIN" -lan -dedicated +set net_port "$PUERTO" +map_rotate >"$LOG" 2>&1 &
echo "Servidor lanzado (pid $!). Conecta con: connect <ip>:$PUERTO"

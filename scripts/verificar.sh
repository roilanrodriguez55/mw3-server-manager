#!/usr/bin/env bash
# Comprueba que todo lo necesario esta en su sitio. No modifica nada. Uso: ./verificar.sh
# Define PLUTONIUM_STORAGE si hay varias instalaciones (ver perfiles.sh).
AQUI=$(cd "$(dirname "$0")" && pwd)
PERFILES=$(cd "$AQUI/../perfiles" && pwd)
ok(){ printf '  [ok]    %s\n' "$1"; }; mal(){ printf '  [FALTA] %s\n' "$1"; FALLOS=$((FALLOS+1)); }; aviso(){ printf '  [aviso] %s\n' "$1"; }
FALLOS=0
STORAGE=$("$AQUI/perfiles.sh" ruta 2>&1) || { echo "$STORAGE"; exit 1; }
PLU=${STORAGE%/storage/iw5}
echo "Plutonium: $PLU"
echo "1) Plutonium"
for f in bin/plutonium-bootstrapper-win32.exe bin/plutonium-launcher-win32.exe games/iw5mp.exe config.json; do
  [ -f "$PLU/$f" ] && ok "$f" || mal "$f (abre el launcher de Plutonium una vez y deja que actualice)"; done
echo "2) Archivos de MW3"
MW3=$(sed -n 's/.*"iw5Path":"\([^"]*\)".*/\1/p' "$PLU/config.json" 2>/dev/null | sed 's/\\\\/\//g; s#^Z:##')
if [ -d "$MW3" ]; then ok "carpeta del juego: $MW3"; else mal "iw5Path de config.json no existe ($MW3): elige la carpeta de MW3 en el launcher"; fi
[ -f "$MW3/main/iw_00.iwd" ] && ok "main/iw_00.iwd" || mal "main/iw_00.iwd (copia incompleta del juego)"
[ -f "$MW3/zone/$(ls "$MW3/zone" 2>/dev/null | head -1)/common_mp.ff" ] && ok "zone/<idioma>/common_mp.ff" || mal "zone/<idioma>/common_mp.ff"
for m in mp_seatown mp_dome mp_bootleg; do ls "$MW3"/zone/*/$m.ff >/dev/null 2>&1 && ok "mapa base $m" || mal "mapa base $m"; done
echo "3) Archivos que Plutonium baja a storage/iw5 (mapas extra y parches)"
for f in zone/plutonium_patch_mp.ff zone/mp_nuked.ff zone/mp_rust.ff zone/mp_nightshift.ff zone/mp_favela.ff mp_nuked.iwd; do
  [ -f "$STORAGE/$f" ] && ok "$f" || aviso "$f no esta (los perfiles 'chico' usan Nuketown, Rust, Skidrow y Favela; abre el launcher para que los baje)"; done
echo "4) Mod de bots"
if [ -f "$STORAGE/z_svr_bots.iwd" ]; then
  ok "z_svr_bots.iwd instalado ($(md5sum < "$STORAGE/z_svr_bots.iwd" | cut -c1-32))"
  unzip -p "$STORAGE/z_svr_bots.iwd" maps/mp/bots/_bot_script.gsc 2>/dev/null | grep -q 'grnd::isingrindzone' \
    && aviso "sin el parche --grnd: puede salir 'unknown function isingrindzone' (bots/BOT_WARFARE_LEEME.md)"
  unzip -p "$STORAGE/z_svr_bots.iwd" maps/mp/bots/_bot_utility.gsc 2>/dev/null | grep -q 'isvalidweaponbuff' \
    && aviso "sin el parche --buff: los bots pueden dar 'script runtime error' al equiparse"
else mal "z_svr_bots.iwd (sin el, los perfiles con bots no tienen bots)"; fi
GSC=$(find "$STORAGE/maps" -name '*.gsc' 2>/dev/null | wc -l)
[ "$GSC" = 0 ] && ok "no hay .gsc sueltos en storage/iw5/maps" || mal "$GSC .gsc sueltos en storage/iw5/maps (error isjuggernautweapon): borralos con el launcher ya abierto"
echo "5) Servidor"
[ -f "$STORAGE/server.cfg" ] && ok "server.cfg (lo escribe el gestor al iniciar un perfil)" || aviso "no hay server.cfg: se crea al activar un perfil"
ls "$PERFILES"/mw3_*.cfg >/dev/null 2>&1 && ok "$(ls "$PERFILES"/mw3_*.cfg | wc -l) perfiles en $PERFILES" || mal "no hay perfiles mw3_*.cfg"
ss -uln 2>/dev/null | grep -q ':5000 ' && aviso "el puerto UDP 5000 ya esta en uso (hay un servidor abierto)" || ok "puerto UDP 5000 libre"
command -v wine >/dev/null || ls "$HOME"/.local/share/Steam/compatibilitytools.d/GE-Proton*/files/bin/wine >/dev/null 2>&1 \
  && ok "wine disponible" || aviso "no encontre wine (servidor.sh necesita WINE=/ruta)"
echo; [ "$FALLOS" = 0 ] && echo "Todo lo imprescindible esta." || echo "$FALLOS cosas imprescindibles faltan."
exit $((FALLOS>0))

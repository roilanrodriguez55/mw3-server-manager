# Scripts (Linux, sin ventana)
Los perfiles estan en `../perfiles/`. Estos scripts los copian, activan y lanzan.

```
cd scripts
./verificar.sh                   # comprueba juego, mapas, bots y puerto sin modificar nada
./perfiles.sh ruta               # muestra que carpeta de storage va a usar
./perfiles.sh listar             # nombres de perfiles
./perfiles.sh instalar           # copia los mw3_*.cfg a storage/iw5/main (por si quieres verlos en LanLauncher)
./perfiles.sh activar tdm_bots   # copia el perfil sobre server.cfg (con respaldo)
./servidor.sh tdm_bots 5000      # activa el perfil y lanza el servidor con Wine, en segundo plano
```
Si encuentran 0 o varias instalaciones de Plutonium (aqui hay Bottles y Bodega), se detienen y listan las rutas. Elige una con
`PLUTONIUM_STORAGE="<ruta>/Plutonium/storage/iw5"`. `servidor.sh` escribe `servidor-<perfil>.log` en esta carpeta; tambien acepta `WINE` y `MW3_DIR`.
Para cerrar el servidor, cierra su ventana o termina su proceso.

## A mano (cualquier emulador o Windows)
1. Copia el `mw3_X.cfg` elegido sobre `storage/iw5/server.cfg` (guarda antes una copia del actual).
2. Cierra cualquier servidor abierto (no relee el cfg en caliente).
3. Lanza: `plutonium-bootstrapper-win32.exe iw5mp "<carpeta de MW3>" -lan -dedicated +set net_port 5000 +map_rotate`
   desde la carpeta de Plutonium (el bootstrapper busca `games\iw5mp.exe` relativo a la carpeta actual).
4. En la consola debe aparecer `rotating to map "<primer mapa de la rotacion>"`.

## Donde esta la carpeta de Plutonium (por emulador)
Todo va en `<prefijo de Wine>/drive_c/users/<usuario>/AppData/Local/Plutonium/storage/iw5`
(en Windows: `%LOCALAPPDATA%\Plutonium\storage\iw5`). Lo que cambia entre emuladores es el prefijo:

| Emulador | Prefijo (ejemplo real de este equipo) |
|---|---|
| Bottles (flatpak) | `~/.var/app/com.usebottles.bottles/data/bottles/bottles/<bottle>` , usuario `steamuser` |
| Bottles (nativo) | `~/.local/share/bottles/bottles/<bottle>` |
| Bodega | `~/.local/share/bodega/bottles/<bottle>/prefix` , usuario `roilan` |
| Wine normal | `~/.wine` |
| Lutris / umu | la carpeta del juego en `~/Games/...` |
| Steam Proton | `~/.local/share/Steam/steamapps/compatdata/<appid>/pfx` |
| Windows | `C:\Users\<usuario>\AppData\Local\Plutonium` |

Si no sabes donde quedo, busca: `find ~ -type d -path '*/Plutonium/storage/iw5' 2>/dev/null`


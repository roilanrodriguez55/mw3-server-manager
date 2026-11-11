# MW3 LAN con Plutonium, LanLauncher y bots

Todo lo necesario para volver a montar un servidor LAN de Modern Warfare 3 (Plutonium IW5) con bots,
perfiles de rotacion y un gestor grafico que inicia el servidor sin LanLauncher. Pensado para Linux con Bottles o Bodega, pero las rutas
de Windows y Wine tambien estan documentadas.

## Estructura
| Carpeta | Contenido |
|---|---|
| `perfiles/` | Los 14 perfiles `mw3_*.cfg` (4 rotaciones normales + 3 de mapas pequenos, con y sin bots), `plantilla.cfg` y `GUIA_CREAR_CONFIGS.md` |
| `scripts/` | `perfiles.sh`, `servidor.sh` y `verificar.sh`: instalar, activar y lanzar sin ventana. Rutas por emulador |
| `gestor-py/` | App de ventana en Python (Tkinter): edita perfiles, inicia y detiene el servidor. Fuente y ejecutables de Linux y Windows |
| `gestor-rs/` | Reescritura en Rust con eframe/egui 0.37 (codigo y README con el estado y lo pendiente) |
| `bots/` | Mod Bot Warfare 2.3.0 (`z_svr_bots.iwd` original), sus instrucciones, `parchar_iwd.py` y `BOT_WARFARE_LEEME.md` |
| `lanlauncher/` | `LanLauncher.exe` v2.0.1 (de JugAndDoubleTap), su actualizador `LLUpdater.exe`, `LAN.ico` y un `LanLauncher.ini.ejemplo` |
| `docs/` | `INSTALACION_DESDE_CERO.md` (guia completa: bottle, juego, Plutonium, mapas, bots, red, sorpresas) y `HISTORIAL.md` |

## LanLauncher incluido
`lanlauncher/LanLauncher.exe` es el programa original, sin cambios (md5 8e7daa4a9caa94e13f4f2f2bfa1fba8f).
Es de JugAndDoubleTap (github.com/JugAndDoubleTap/LanLauncher), con sus propios terminos. Copia la carpeta `lanlauncher/` a cualquier
lugar y ejecuta el `.exe` con Bottles o Bodega ("Ejecutar ejecutable"), o con Wine. Al primer arranque crea su `LanLauncher.ini`
(guia en `LanLauncher.ini.ejemplo`): `plutonium folder` y `modernwarfare 3 folder` deben apuntar a las carpetas dentro del bottle,
por ejemplo `C:/Users/steamuser/AppData/Local/Plutonium` y la carpeta de MW3. `LLUpdater.exe` es su actualizador: si se
ejecuta, puede reemplazar el `.exe` por una version mas nueva desde internet.

## Lo unico que no se puede incluir
- Plutonium (https://plutonium.pw): requiere tu cuenta y se actualiza solo. Ya debe estar instalado en el bottle y funcionando.
- Los archivos de MW3: son del juego y pesan decenas de GB. Plutonium necesita una copia que reconozca.
Con eso mas este repositorio no hace falta descargar nada mas.

Primera vez o instalacion nueva: lee `docs/INSTALACION_DESDE_CERO.md` y ejecuta `scripts/verificar.sh`, que comprueba sin modificar nada que
las piezas esten en su sitio.

## Pasos rapidos
1. Instalar el mod de bots (`bots/BOT_WARFARE_LEEME.md`): copiar `z_svr_bots.iwd` a `storage/iw5` de Plutonium,
   con el juego cerrado. En este equipo hizo falta el parche (`python3 bots/parchar_iwd.py ...`).
2. Abrir el gestor: `gestor-py/gestor-servidor-linux` (o el `.exe` dentro del bottle). Elegir un perfil de la lista, editarlo si
   quieres, y pulsar "Iniciar con el perfil seleccionado". Sin ventana: `cd scripts && ./servidor.sh chico_ffa_nobots 5000`.
3. Conectar: misma PC `connect 127.0.0.1:5000`, otra PC `connect <ip de la red>:5000`.
4. Si algo falla: `docs/HISTORIAL.md` y los problemas conocidos de `bots/BOT_WARFARE_LEEME.md`.

LanLauncher sirve para lanzar el servidor, pero no para elegir el perfil: ignora la seleccion (ver `docs/HISTORIAL.md`, punto 11).

## Lo que no se probo
Que todos los modos funcionen en todos los mapas, que los `scr_<modo>_...` de duracion y puntuacion los tome Plutonium
desde el cfg, que el `.exe` del gestor inicie el servidor dentro de un bottle, y el parche `--buff` instalado.

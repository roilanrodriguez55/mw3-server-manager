# Bot Warfare para MW3 en Plutonium (z_svr_bots.iwd)

Los bots de los servidores y de las partidas privadas salen de este mod. Sin el, los perfiles de
`perfiles/` (`bots_manage_fill`, `bots_skill`, etc.) no hacen nada, porque esas variables
las lee el mod. Version: IW5 Bot Warfare 2.3.0 (mod GSC de ineedbots para PlutoniumIW5).
Proyecto: https://github.com/ineedbots/iw5_bot_warfare

## Contenido de esta carpeta
| Archivo | Que es |
|---|---|
| `z_svr_bots.iwd` | El mod original, sin tocar (md5 f2a449f694af73a001af7b986a08ca24) |
| `install.bat` | Instalador original de Windows: copia el `.iwd` a `%LOCALAPPDATA%\Plutonium\storage\iw5\` |
| `README.txt` | Documentacion original del autor (menu, creditos, cambios de la 2.3.0) |
| `parchar_iwd.py` | Genera una copia parcheada del `.iwd` (ver "Parches") |
| `BOT_WARFARE_LEEME.md` | Este archivo |

## Instalacion
Es un solo archivo. Cualquiera de las dos formas:
- Windows: ejecuta `install.bat`.
- Linux con Bottles, Bodega, Wine, etc.: copia `z_svr_bots.iwd` (o la version parcheada) a
  `<prefijo>/drive_c/users/<usuario>/AppData/Local/Plutonium/storage/iw5/`.
  Las rutas por emulador estan en `scripts/README.md`. Ejemplo con Bottles flatpak:
  ```
  cp z_svr_bots.iwd ~/.var/app/com.usebottles.bottles/data/bottles/bottles/Games/drive_c/users/steamuser/AppData/Local/Plutonium/storage/iw5/
  ```
Hay que hacerlo en cada instalacion de Plutonium que uses (Bottles, Bodega, otra PC si hospeda ella). Un cliente que solo se
conecta a tu servidor no lo necesita: los bots corren en el servidor.

No cambies ni reemplaces el `.iwd` con el juego o el servidor abiertos: el motor lo detecta y se cierra con
`Sys_Error: IWD modified`. Cierra todo, copia y vuelve a abrir.

## Uso en partida privada
Se abre el menu con la tecla de la vision nocturna (por defecto N), se navega con WASD y se elige con
espacio. Para servidores dedicados se controla por variables en el cfg (guia: `perfiles/GUIA_CREAR_CONFIGS.md`,
seccion Bots).

## Parches que necesito en este equipo
El mod 2.3.0 espera scripts del juego mas nuevos que los de esta instalacion de MW3 (repack con
textos en espanol, archivos de 2015). Al probarlo salieron errores `unknown function` y `script runtime error`.
No lo he podido confirmar como causa unica, es la explicacion mas probable. Los parches los aplica `parchar_iwd.py`:

| Parche | Que arregla | Estado en este equipo |
|---|---|---|
| `--grnd` | `LinkFile: ... unknown function isingrindzone` al crear la partida: el mod llamaba a `grnd::isingrindzone()`, que el juego base no tiene. Lo cambia por la variable `ingrindzone` del jugador (4 llamadas) | Aplicado en el `.iwd` instalado del bottle Games |
| `--buff` | `script runtime error ... isvalidweaponbuff` por cada bot al darle equipamiento: el juego la define con otros argumentos. Reemplaza las 2 llamadas por `true` | Preparado, no instalado |

Generar y probar:
```
python3 parchar_iwd.py z_svr_bots.iwd z_svr_bots_parcheado.iwd            # los dos parches
python3 parchar_iwd.py z_svr_bots.iwd z_svr_bots_parcheado.iwd --grnd     # solo uno
```
Despues copialo a `storage/iw5/` con el nombre `z_svr_bots.iwd` (juego cerrado). Con los dos parches el md5 queda
5918c6ac832def18296107b3cf3a824b, con solo `--grnd` 9d36a3c9fe76be9cc96b60f54fc9cff8 (el que esta instalado).
Los parches no cambian el comportamiento de los bots en Drop Zone ni la validacion de mejoras de arma mas alla de eso.

## Problemas conocidos
- `LinkFile: ... isjuggernautweapon`: aparece si en `storage/iw5/maps/mp/` hay `.gsc` sueltos
  (`_killstreaks`, `_class`, `_gamelogic`, `_helicopter`, `_nuke`, `_callbacksetup`, `_music_and_dialog`, `_dev`,
  `infect`, `dd`, `mp_rust`, `mp_rust_fx`). El launcher de Plutonium los descarga otra vez cada vez que lo abres.
  Solucion: con el launcher abierto, borra esos `.gsc` y entonces pulsa Jugar. Si los dejas, el juego carga esos
  y no los del juego base. En nuestras pruebas, un servidor lanzado desde LanLauncher no los regenero.
- Bots que aparecen pero sin equipamiento bueno o con errores en consola: falta el parche `--buff`.
- Los bots necesitan puntos de ruta (waypoints) por mapa. El `.iwd` trae los de los mapas base y los de Plutonium
  (Nuketown, Rust, Skidrow, Favela, Highrise). En un mapa sin waypoints no se que hacen (no lo probe).

## Si la version del juego base es otra
Si usas un MW3 actualizado a la version que espera Plutonium r5369, estos parches no hacen falta: usa el `.iwd` original
y borra los `.gsc` sueltos solo si el error aparece. No probe esa instalacion.

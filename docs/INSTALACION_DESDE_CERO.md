# Montar MW3 LAN con bots desde cero

Guia para repetir todo el montaje sin sorpresas: el juego, Plutonium, los mapas, el mod de bots, los perfiles
y el servidor. Cada paso dice como comprobar que quedo bien. Los datos de "en este equipo" son los reales de la
instalacion que funciono el 9 y 10 de noviembre de 2026. Lo que no se probo esta marcado como tal.

Para comprobar casi todo de golpe (no modifica nada):
```
cd scripts && ./verificar.sh
```

## 0. Versiones con las que funciono
| Pieza | Version o dato |
|---|---|
| Plutonium | revision r5369, juego `iw5mp` |
| MW3 | repack con textos en espanol, archivos base de 2015 (`main/iw_00..22.iwd`, `zone/spanish/`) |
| Emulacion | Bottles (flatpak), bottle `Games`, 64 bits, entorno Gaming, runner `GE-Proton11-11`, DXVK 3.1.1, D7VK 2.3 |
| Tambien existe | Bodega, con su propio prefijo y su propia copia de Plutonium. Esta documentado pero no se uso para jugar |
| Mod de bots | Bot Warfare 2.3.0, con los parches de `bots/` |
| Red | PC del servidor en `192.168.8.x`, puerto UDP 5000 |

## 1. Lo que hay que tener antes de empezar
1. Linux con Bottles, Bodega, Wine o Proton (o Windows, y entonces se salta la parte del bottle).
2. Una copia completa de MW3 (la de PC, version multijugador). Plutonium no la incluye y el repo tampoco: pesa decenas de GB.
3. Una cuenta de Plutonium (https://plutonium.pw). El launcher la pide al primer arranque y guarda un token en `config.json`.
   No copies ese archivo a ningun sitio.
4. Este repositorio completo. Trae LanLauncher, el mod de bots y su parche, los perfiles y el gestor de servidor.

## 2. El bottle (solo Linux)
1. Crea un bottle de 64 bits con entorno Gaming. Aqui se llama `Games` y usa el runner `GE-Proton11-11`.
2. Descarga el launcher `plutonium.exe` desde plutonium.pw (en este equipo esta en `~/Apps/Plutonium/plutonium.exe`, 4.8 MB) y
   agregalo al bottle como programa para poder ejecutarlo.
3. Comprobacion: al ejecutarlo se abre la ventana del launcher de Plutonium.

Despues del primer arranque, Plutonium crea esta carpeta dentro del bottle (la llamamos `<Plutonium>`):
`<prefijo>/drive_c/users/<usuario>/AppData/Local/Plutonium`. En Bottles flatpak el prefijo es
`~/.var/app/com.usebottles.bottles/data/bottles/bottles/<bottle>`, y el usuario de Wine fue `steamuser`.
Las rutas de Bodega, Wine, Lutris, Proton y Windows estan en `scripts/README.md`.

## 3. Los archivos del juego
1. Deja la carpeta de MW3 en una ruta sin sorpresas. Aqui: `~/Documents/Games/Call of Duty Modern Warfare 3`.
2. En el launcher de Plutonium, elige IW5 (Modern Warfare 3) y selecciona esa carpeta. Plutonium guarda la ruta en
   `<Plutonium>/config.json` como `iw5Path` (aqui `Z:\home\roilan\Documents\Games\Call of Duty Modern Warfare 3`; `Z:` es la raiz de Linux
   vista desde Wine).
3. La carpeta debe tener al menos `main/iw_00.iwd` ... `iw_23.iwd`, los `localized_<idioma>_*.iwd` y `zone/<idioma>/` con
   `common_mp.ff` y los mapas. `verificar.sh` lo comprueba.

Sorpresa conocida: los scripts del juego base de este equipo parecen mas viejos que lo que espera Plutonium r5369 (faltan funciones
que el mod y los scripts de Plutonium llaman). Es la explicacion probable, no esta confirmada, y por eso hicieron falta los parches del mod de
bots (apartado 6). No se comprobo con un MW3 actualizado.

## 4. Mapas
Hay dos grupos, y los dos tienen que estar en cada PC que juegue.

| Grupo | Mapas | Donde viven | Quien los pone |
|---|---|---|---|
| Base (16) | mp_alpha Lockdown, mp_bootleg Bootleg, mp_bravo Mission, mp_carbon Carbon, mp_dome Dome, mp_exchange Downturn, mp_hardhat Hardhat, mp_interchange Interchange, mp_lambeth Fallen, mp_mogadishu Bakaara, mp_paris Resistance, mp_plaza2 Arkaden, mp_radar Outpost, mp_seatown Seatown, mp_underground Underground, mp_village Village | `<MW3>/zone/<idioma>/mp_*.ff` | Tu copia del juego |
| Plutonium | mp_nuked Nuketown, mp_rust Rust, mp_nightshift Skidrow, mp_favela Favela, mp_highrise Highrise (y mp_test) | `<Plutonium>/storage/iw5/zone/*.ff` y `storage/iw5/mp_*.iwd` | El launcher de Plutonium al actualizar |

- En este equipo los archivos de los mapas Plutonium aparecieron al abrir el launcher (fechas de la misma sesion). Se espera lo mismo en
  la otra PC, pero no se comprobo.
- Los perfiles normales (`tdm`, `dom`, `ffa`, `mixto`) usan solo mapas base. Los `chico_tdm` y `chico_ffa` incluyen Nuketown,
  Rust, Skidrow y Favela. `chico_dom` usa solo mapas base.
- Si un jugador no tiene un mapa, no puede cargarlo. Para quitar dudas usa los perfiles normales.
- No sabemos si todos los modos funcionan en todos los mapas. En Nuketown y Rust solo pensamos en `war` y `dm`.

Comprobacion: `./verificar.sh` lista los mapas base de prueba y los de Plutonium.

## 5. Dentro del juego y de Plutonium
Antes de montar el servidor, comprueba que el juego funciona solo:
1. Abre el launcher, elige IW5 y pulsa jugar (multijugador). Entra con tu cuenta.
2. Crea una partida privada y carga un mapa. Si esto falla, el problema es el juego o el bottle, no el servidor. Los errores de
   `LinkFile ... unknown function` de esta etapa estan en el apartado 8.
3. Nombre de jugador: LanLauncher lo pasa como `+name <username>` (campo `username` de `LanLauncher.ini`). Con el launcher de Plutonium
   no comprobamos de donde sale, probablemente de tu cuenta.

Consolas, para conectar a un servidor:
- Plutonium abre una ventana de texto aparte, "Plutonium r5369", con una linea `Plutonium r5369 >` para escribir.
  Ahi funciono `connect 127.0.0.1:5000` en este equipo.
- La consola del propio juego (tecla bajo Esc, segun el teclado) borraba lo que se escribia al llegar a cierto punto. No se encontro
  la causa. Si te pasa, usa la ventana externa, o un archivo `c.cfg` en `storage/iw5` con la linea `connect <ip>:5000` y escribe `exec c.cfg`
  (esta variante no llego a probarse del todo).
- Usa `127.0.0.1` para la misma PC y la IP de la red para otra. `0.0.0.0` no vale.

Mensajes que salen siempre en la consola y no impiden jugar: `Unknown command "con_showchannel"`,
`game_suicidepointloss`, `xblive_rankedmatch`, `xblive_privateserver`, `"24" is not a valid value for dvar "sv_maxclients"`,
`Could not find publisher file "motd-spanish.txt"` y `Hitch warning` de varios segundos al cargar.

## 6. El mod de bots
Sin `z_svr_bots.iwd` en `storage/iw5`, los perfiles con bots no ponen ninguno. Instalacion, parches y problemas conocidos:
`bots/BOT_WARFARE_LEEME.md`. Resumen:
1. Con el juego, el servidor y el launcher cerrados, copia `bots/z_svr_bots.iwd` a `<Plutonium>/storage/iw5/`.
2. En este equipo hace falta el parche: `python3 bots/parchar_iwd.py bots/z_svr_bots.iwd salida.iwd` y copiar `salida.iwd` con el
   nombre `z_svr_bots.iwd`. `--grnd` evita el error `isingrindzone` al crear la partida y `--buff` evita los `script runtime error` de
   los bots. Este equipo tuvo solo `--grnd` instalado.
3. No cambies nunca el `.iwd` con el juego o el servidor abiertos: se cierra con `IWD modified`.
4. Si al crear la partida sale `LinkFile ... isjuggernautweapon`, hay `.gsc` sueltos en `storage/iw5/maps/mp/`. El launcher
   los vuelve a descargar cada vez que se abre. Borralos con el launcher ya abierto y despues pulsa jugar. El servidor lanzado con el gestor no los
   regenero.

## 7. Perfiles y servidor
1. Abre el gestor: `gestor-py/gestor-servidor-linux` (o `gestor-py/gestor-servidor.exe` dentro del bottle; la version en Rust esta en `gestor-rs/`). Si no encuentra `storage/iw5`,
   usa "Ajustes de rutas".
2. Elige un perfil de la lista, editalo si quieres y pulsa "Iniciar con el perfil seleccionado".
3. Comprobacion: abajo, en la consola, debe salir `rotating to map "<primer mapa de la rotacion>"`.
4. Sin ventana: `cd scripts && ./servidor.sh <perfil> 5000`. Con otra instalacion de Plutonium:
   `PLUTONIUM_STORAGE=<ruta>/Plutonium/storage/iw5 ./servidor.sh <perfil>`.
5. Para cambiar de perfil: cierra el servidor y arranca otro. Un servidor no relee su cfg en caliente.

Por que se usa el gestor y no LanLauncher: LanLauncher envia `+exec <perfil>` pero el servidor lo ignora y lee siempre
`storage/iw5/server.cfg`. LanLauncher sigue sirviendo para lanzar un servidor, y el gestor, `servidor.sh` y `perfiles.sh activar`
copian el perfil sobre `server.cfg` antes de arrancar.

Lo que se copia a la carpeta de Plutonium:
| Archivo | Para que | Imprescindible |
|---|---|---|
| `storage/iw5/z_svr_bots.iwd` | bots | solo si quieres bots |
| `storage/iw5/server.cfg` | lo que el servidor lee al arrancar (lo escribe el gestor) | si, pero lo crea la herramienta |
| `storage/iw5/main/*.cfg` | solo para que salgan en la lista de LanLauncher | no |
| `*.dsr`, `default.dspl`, `c.cfg`, `main_respaldo/` | sobras de pruebas de esta sesion | no. Los perfiles usan rotacion en `sv_maprotation`, no los `.dsr` |

## 8. Red y otras PC
1. El servidor escucha en `0.0.0.0:5000` (UDP), o sea en todas las interfaces. Comprobacion: `ss -uln | grep :5000`.
2. IP del servidor: `ip -4 addr`. En este equipo la WiFi era `192.168.8.151`. Si el router la cambia, el comando de conexion cambia.
3. Firewall: en este equipo `ufw` estaba con `ENABLED=no` y `firewalld` inactivo. Si en el tuyo hay uno activo, abre UDP 5000.
4. Router: si el router aisla a los clientes de la WiFi ("AP isolation"), las PC no se ven entre si.
5. Otra red o VPN: se vio una conexion desde `192.168.2.204`, otra subred, asi que no hace falta estar en la misma. Tailscale
   (`100.x.x.x:5000`) es una opcion, sin probarla con este juego.
6. La otra PC necesita: Plutonium IW5 de la misma revision, MW3 completo y los mapas que use el perfil (apartado 4). No necesita
   el mod de bots.
7. Conectar: `connect <ip>:5000` en la consola de Plutonium.
8. La lista de servidores LAN del juego no mostro el servidor (puerto distinto al predeterminado). Se entra con `connect`.

## 9. Orden recomendado de una instalacion nueva
1. Bottle y launcher de Plutonium (apartado 2). Verifica que se abre.
2. Archivos del juego y ruta en Plutonium (apartado 3).
3. Abre el launcher hasta que termine de descargar y entra a una partida privada (apartado 5).
4. Cierra todo. Instala el mod de bots y el parche (apartado 6).
5. Abre otra vez el launcher, borra los `.gsc` sueltos si aparecieron, y crea una partida privada con bots para probar.
6. `./verificar.sh`, despues el gestor y un perfil sin bots (apartado 7).
7. Prueba con bots, y despues desde otra PC (apartado 8).

## 10. Sorpresas y su arreglo
| Sintoma | Causa | Arreglo |
|---|---|---|
| `LinkFile ... isjuggernautweapon` al crear partida | `.gsc` sueltos en `storage/iw5/maps/mp/` que el launcher vuelve a bajar | Borrarlos con el launcher ya abierto |
| `LinkFile ... isingrindzone` | El mod llama a una funcion de Drop Zone que el juego base no tiene | Parche `--grnd` |
| `script runtime error ... isvalidweaponbuff` | El juego la define con otros argumentos | Parche `--buff` |
| `Sys_Error: IWD modified` | Se cambio un `.iwd` con el juego abierto | Cerrar todo antes de copiar |
| El servidor arranca pero `server is not running` | No tiene mapa: sin `sv_maprotation` o sin `+map_rotate` | Usar un perfil del repo y el gestor |
| Siempre carga el mismo perfil en LanLauncher | El servidor lee `server.cfg` e ignora `+exec` | Usar el gestor o `servidor.sh` |
| `executable ... games\iw5mp.exe not found!` | El bootstrapper busca ese archivo relativo a la carpeta actual | Ejecutarlo desde la carpeta de Plutonium (el gestor y `servidor.sh` ya lo hacen) |
| Dos servidores a la vez fallan | Chocan por el puerto 5000 | Cerrar el viejo o usar otro puerto |
| La consola del juego borra el texto | Sin causa conocida | Usar la ventana externa de Plutonium |
| `Could not find publisher file motd-spanish.txt` | Archivos del servidor de Plutonium que no existen en LAN | Ignorarlo |
| Los bots no aparecen | Sin `z_svr_bots.iwd`, o el perfil esta en "sin bots" | Instalar el mod y usar un perfil `_bots` |

## 11. Lo que no se comprobo
- Instalacion con un MW3 actualizado, o con otro idioma.
- Bodega para jugar, y el `.exe` del gestor iniciando el servidor desde dentro del bottle.
- Que otra PC descargue sola los mapas de Plutonium, y que entre al servidor con cada perfil.
- Que el servidor rote de mapa al terminar una partida, y que funcionen las variables de limite de tiempo y puntos escritas en el cfg.
- El parche `--buff` instalado en el bottle.
- Los datos de cuenta, y como crear la cuenta de Plutonium.

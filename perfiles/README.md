# Perfiles de servidor LAN para MW3 (Plutonium)

14 perfiles: 7 rotaciones, cada una con bots y sin bots.

| Perfil | Modo y mapas | Bots |
|---|---|---|
| tdm_bots / tdm_nobots | Team Deathmatch, 12 mapas | mantiene 10 jugadores |
| dom_bots / dom_nobots | Dominacion, 12 mapas | mantiene 12 |
| ffa_bots / ffa_nobots | Todos contra todos, 10 mapas | mantiene 8 |
| mixto_bots / mixto_nobots | TDM, Dominacion, HQ (koth) y Buscar y Destruir, 3 mapas cada uno | mantiene 10 |
| chico_tdm_bots / chico_tdm_nobots | TDM en mapas pequenos (Nuketown, Rust, Dome, Skidrow, Favela, Bootleg, Village), max 8 jugadores | mantiene 6 |
| chico_ffa_bots / chico_ffa_nobots | Todos contra todos, mismos mapas, max 8 | mantiene 6 |
| chico_dom_bots / chico_dom_nobots | Dominacion en Dome, Bootleg, Village y Underground, max 8 | mantiene 6 |

Los perfiles normales usan solo mapas del juego base, para que otra PC los tenga sin descargas extra. Los `chico_tdm` y `chico_ffa` incluyen Nuketown, Rust, Skidrow y Favela, que Plutonium descarga aparte (ya estan en tu `storage/iw5`); la otra PC los necesita igual. `chico_dom` usa solo mapas base.
Con bots: `bots_manage_fill N` mantiene N jugadores en total (humanos + bots) y
`bots_manage_fill_kick 1` saca bots cuando entran humanos. Para cambiar la cantidad
edita esa linea. Sin bots: fill y add en 0 (el mod queda cargado pero sin agregar ninguno).

## Requisito: el mod de bots
Los perfiles con bots dependen de `z_svr_bots.iwd` (Bot Warfare 2.3.0), instalado en `storage/iw5`. Sin el, `bots_manage_fill` y las demas variables `bots_*` no hacen nada. Como instalarlo, los parches necesarios en este equipo y los errores conocidos estan en `../bots/BOT_WARFARE_LEEME.md`.

## Crear tus propios perfiles
Con una de las apps (carpeta `gestor-py/` hoy, `gestor-rs/` la version en Rust) o a mano: ver `GUIA_CREAR_CONFIGS.md` y `plantilla.cfg`.
Los archivos deben llamarse `mw3_<nombre>.cfg` para que los scripts y las apps los encuentren.

## Como se usan
Estos archivos no hacen nada solos. Hay que activarlos (copiarlos sobre `storage/iw5/server.cfg`) y arrancar el servidor:
- Con una app: `gestor-py/README.md`.
- Con scripts: `scripts/README.md`.

## Conectar
- Misma PC: `connect 127.0.0.1:5000`
- Otra PC de la red: `connect <IP de esta PC>:5000` (ahora 192.168.8.151)

## Notas
- Dos servidores a la vez necesitan puertos distintos (campo Port number).
- El cliente de la otra PC no necesita `z_svr_bots.iwd`, los bots corren en el servidor.
- El mod `z_svr_bots.iwd` debe estar en `storage/iw5` y no se puede cambiar mientras el juego esta abierto (`IWD modified`).

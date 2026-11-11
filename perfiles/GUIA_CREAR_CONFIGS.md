# Guia para crear tus propios perfiles de servidor

Un perfil es un archivo de texto con lineas `set <variable> <valor>`. Se pueden crear y editar con una de las apps (`gestor-py/` o `gestor-rs/`), o a mano: parte de `plantilla.cfg`,
copiala con el nombre `mw3_<tu_nombre>.cfg` en esta carpeta y activala con `scripts/perfiles.sh activar` o `scripts/servidor.sh` (ver `../scripts/README.md`). Los valores entre comillas son texto, los demas numeros.
Un servidor ya encendido no relee el archivo: hay que cerrarlo y lanzarlo de nuevo.

## 1. Rotacion de mapas
```
set sv_maprotation "gametype war map mp_seatown map mp_dome gametype dom map mp_carbon"
set g_gametype war
```
- `gametype X` cambia el modo y vale para todos los `map` que le sigan, hasta el proximo `gametype`.
- Al llegar al final vuelve a empezar.
- `g_gametype` conviene que sea el modo de la primera entrada.

Modos (nombres de los `.dsr` del juego):

| Codigo | Modo | Codigo | Modo |
|---|---|---|---|
| war | Team Deathmatch | koth | HQ |
| dm | Todos contra todos | ctf | Captura la bandera |
| dom | Dominacion | tdef | Team Defender |
| sd | Buscar y Destruir | sab | Sabotaje |
| conf | Kill Confirmed | dd | Demolicion |
| grnd | Drop Zone | gun | Gun Game |
| infect | Infectados | oic | Una en la recamara |
| jugg | Juggernaut | tjugg | Team Juggernaut |

Mapas del juego base: mp_alpha, mp_bootleg, mp_bravo, mp_carbon, mp_dome, mp_exchange,
mp_hardhat, mp_interchange, mp_lambeth, mp_mogadishu, mp_paris, mp_plaza2, mp_radar,
mp_seatown, mp_underground, mp_village.
Descargados por Plutonium (los tiene tu `storage/iw5`, la otra PC los necesita): mp_nuked,
mp_rust, mp_nightshift, mp_favela, mp_highrise.

Lo que no sabemos: que todos los modos funcionen en todos los mapas (en Nuketown o Rust solo
probamos pensar en war y dm). Si un mapa no carga con un modo, quitalo de esa rotacion.

## 2. Duracion y puntuacion
La forma es `scr_<modo>_<opcion>`, con el codigo de modo de la tabla:
```
set scr_war_timelimit 10      // minutos, 0 = sin limite
set scr_war_scorelimit 7500   // puntos
set scr_sd_roundlimit 6       // rondas (modos por rondas)
set scr_sd_winlimit 4         // rondas para ganar
set scr_war_numlives 0        // vidas por ronda, 0 = ilimitadas
```
Esos nombres salen del codigo de los modos del juego (`registerTimeLimitDvar`, `registerScoreLimitDvar`,
`registerRoundLimitDvar`, `registerWinLimitDvar`, `registerNumLivesDvar`). No probamos que
Plutonium los tome desde el cfg: si no cambian nada, ponlos en la consola del servidor antes
de cargar el mapa y avisame.

Valores por defecto de los `.dsr` del juego, como referencia:

| Modo | timelimit | scorelimit | numlives |
|---|---|---|---|
| war | 10 | 7500 | 0 |
| dom | 2.5 | 500 | 0 |
| sd | 0 | 4 | 1 |

En war la puntuacion son puntos y no bajas (7500 es el valor original).

Otras variables que aparecen en el codigo del juego, sin comprobar su efecto: `scr_game_hardpoints`
(rachas), `scr_game_perks`, `scr_game_killstreakdelay`, `scr_player_forcerespawn`, `scr_team_fftype`
(fuego amigo).

## 3. Bots (Bot Warfare)
Cantidad:
```
set bots_manage_add 4          // agrega 4 bots al arrancar y ya
set bots_manage_fill 10        // mantiene 10 jugadores en total (humanos + bots)
set bots_manage_fill_mode 0    // 0 cuenta a todos, 1 solo bots, 2 y 3 mantienen por equipo
set bots_manage_fill_kick 1    // saca bots cuando hay de mas
set bots_manage_fill_spec 1    // cuenta a quien este en espectador (1 si, 0 no)
set bots_manage_fill_watchplayers 0  // 1 = solo hay bots si hay humanos, y se van al irse ellos
```
Sin bots: `bots_manage_add 0` y `bots_manage_fill 0`.

Dificultad:
```
set bots_skill 0   // 0 aleatorio entre min y max, 1 facil ... 7 dificil, 8 personalizada, 9 totalmente aleatorio
set bots_skill_min 1
set bots_skill_max 7
```
- Para todos faciles: `bots_skill 1`. Para todos dificiles: `bots_skill 7`.
- Personalizada (`bots_skill 8`): `bots_skill_axis_hard`, `bots_skill_axis_med`, `bots_skill_allies_hard`
  y `bots_skill_allies_med` son cuantos bots de cada nivel hay en cada equipo, los demas quedan faciles.

Equipo:
```
set bots_team autoassign   // autoassign, allies, axis o spectator
set bots_team_amount 0     // cuantos bots forzar al equipo axis (0 = no forzar)
set bots_team_mode 0       // 1 = al repartir equipos cuenta solo a los bots
```
Ejemplo para humanos contra bots: `bots_team axis`.

Esos nombres y valores salen del codigo del mod (`_bot.gsc` v2.3.0). El mod tiene mas opciones
(`bots_play_*`, `bots_loadout_*`, etc.) que no cubro aqui.

## 4. Datos del servidor
```
set sv_hostname "Nombre en el listado"
set sv_maxclients 16      // 1 a 18
set g_password ""         // contrasena para entrar
set rcon_password ""      // administrar desde la consola de un cliente
set g_allowVote 1
set g_inactivity 90       // segundos antes de echar a quien no se mueve
```

## 5. Receta rapida
1. `cp plantilla.cfg mw3_mi_perfil.cfg`
2. Cambia `sv_hostname`, `sv_maprotation`, `g_gametype` y los `scr_<modo>_...` al modo que uses.
3. Ajusta los bots (seccion 3).
4. Cierra el servidor actual, elige el perfil en el gestor y pulsa "Iniciar" (o `scripts/servidor.sh mi_perfil`).
5. Mira la consola del servidor: el nombre debe ser el nuevo y el mapa el primero de la rotacion.

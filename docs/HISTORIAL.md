# Historial de lo que hubo que resolver (noviembre 2026)

Sirve para no repetir el camino si vuelves a armarlo. Todo se hizo en un bottle de Bottles (flatpak)
con Plutonium IW5 r5369 y un MW3 repack con textos en espanol (archivos base de 2015).

1. Instalar Bot Warfare: `install.bat` solo copia `z_svr_bots.iwd` a `storage/iw5`. En Bottles es copiar a mano.
2. Error al crear la partida privada: `LinkFile: ... isjuggernautweapon`. Habia 12 `.gsc` sueltos en
   `storage/iw5/maps/mp/` que Plutonium carga en lugar de los del juego. Se borran con el launcher ya abierto
   (lo vuelve a descargar cada vez que se abre).
3. Despues: `LinkFile: ... isingrindzone` (el mod llama a una funcion de Drop Zone que el juego base no tiene).
   Parche `--grnd` de `bots/parchar_iwd.py`.
4. `IWD modified` al cambiar el `.iwd` con el juego abierto: cerrar todo antes de copiar.
5. Servidor LAN con LanLauncher: la lista "Available configs" mostraba los `.cfg` de `storage/iw5/main`, que estaba vacia.
6. El servidor arrancaba pero sin mapa (`server is not running`). El `+exec` del cfg elegido no parecia ejecutarse y solo se
   leia `storage/iw5/server.cfg`, y `+map_rotate` necesita `sv_maprotation`. Perfil con rotacion clasica
   `gametype war map ...` y `set g_gametype`.
7. Dos servidores a la vez chocan por el puerto 5000: cierra el viejo antes de lanzar otro.
8. Conectarse: la lista LAN del juego no encontro el servidor (puerto 5000 no es el predeterminado). Se conecta con
   `connect <ip>:5000` por la consola. La consola del juego borraba lo escrito, y se uso un `c.cfg` con `exec c.cfg`
   o la consola externa de Plutonium. Para otra PC: la IP de la red local.
9. Bots con errores `script runtime error ... isvalidweaponbuff`: parche `--buff`, preparado y sin instalar.
10. Servidor dedicado: los `server.cfg`, `.dspl` y `.dsr` de la carpeta `cod3` son el paquete base del juego (ya
    estan en `players2` del juego). La playlist original tenia `*,TDEF_default,1000`, que hacia casi siempre Team Defender.

Causa probable de 2, 3 y 9: los scripts del juego base son mas viejos que los que espera Plutonium r5369 y el mod 2.3.0.
No esta confirmado.

11. LanLauncher "ignora" el perfil elegido: pasa `+exec <perfil>` pero el servidor solo lee `storage/iw5/server.cfg`
    (probado con el perfil en `main/` y en la raiz de `storage/iw5`). Solucion: copiar el perfil sobre `server.cfg` antes de
    arrancar. Lo hacen `scripts/perfiles.sh activar`, `scripts/servidor.sh` y las apps de ventana.
12. Lanzar el servidor sin LanLauncher con Wine: el bootstrapper busca `games\iw5mp.exe` relativo a la carpeta actual, asi que hay que
    hacer `cd` a la carpeta de Plutonium antes. Con eso, `plutonium-bootstrapper-win32.exe iw5mp "<MW3>" -lan -dedicated
    +set net_port 5000 +map_rotate` rota al primer mapa del perfil activo.


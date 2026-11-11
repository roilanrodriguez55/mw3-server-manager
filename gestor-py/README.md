# Gestor de servidor en Python (app de ventana)
Una sola aplicacion con ventana para todo: lista los perfiles de una carpeta, los carga, los edita, los guarda
y inicia o detiene el servidor del perfil seleccionado, sin LanLauncher.

| Archivo | Para que |
|---|---|
| `gestor-servidor-linux` | Linux nativo: `./gestor-servidor-linux` |
| `gestor-servidor.exe` | Windows, o dentro de Bottles/Bodega/Wine ("Ejecutar ejecutable") |
| `gestor_servidor.py` | Codigo fuente (Python 3 con Tkinter): `python3 gestor_servidor.py` |

Que hace:
- Lista a la izquierda: los `.cfg` de la carpeta de perfiles (por defecto `../perfiles`, la del repo). Botones: Nuevo, Importar
  (copia uno o varios `.cfg` a la carpeta), Abrir (cualquier `.cfg` de cualquier sitio), Duplicar, Eliminar y Carpeta (cambia la
  carpeta de perfiles). Un triangulo marca el perfil que corre.
- Pestañas de edicion: General (nombre, jugadores, contrasenas, voz, votos, inactividad, baneo, duracion, puntos, rondas, vidas,
  fuego amigo, rachas), Rotacion (modo y mapa, subir/bajar/quitar), Bots (cantidad, relleno, dificultad, equipos y los 12
  `bots_play_*`) y Texto del archivo (lineas extra y el texto completo, editable a mano).
- Guardar: escribe `mw3_<nombre>.cfg` en la carpeta de perfiles. Si cambias el nombre, guarda un archivo nuevo. Pregunta antes de
  sobrescribir y al cerrar o cambiar de perfil con cambios sin guardar.
- Cargar un perfil lo deja intacto hasta que cambies algo del formulario. Lo que el formulario no conoce (otras variables, `exec`,
  etc.) se conserva en "lineas extra". Al regenerar el texto se pierden los comentarios del archivo original. Si editas el
  texto a mano, el formulario deja de actualizarlo hasta que pulses "Regenerar" o "Leer el texto".
- Iniciar: copia el perfil guardado sobre `storage/iw5/server.cfg` (guarda antes `server.cfg.bak-FECHA`) y lanza
  `plutonium-bootstrapper-win32.exe iw5mp <MW3> -lan -dedicated +set net_port <puerto> +map_rotate`. En Linux usa Wine con el
  prefijo que deduce de la ruta de storage. Se niega a arrancar si el puerto UDP ya esta en uso.
- Detener: cierra solo el servidor que inicio esa misma ventana. Si lo abriste de otra forma, cierralo desde su ventana.
- Abajo: las ultimas lineas del `console.log` de Plutonium, para ver `rotating to map ...`.
- Ajustes de rutas: carpeta `storage/iw5` (se busca sola en Bottles, Bodega, Wine, Lutris y Proton), carpeta de MW3 (por defecto
  la de `config.json` de Plutonium) y el programa `wine`. Se guardan en `gestor_servidor.json`, junto al ejecutable.

Por que no se usa LanLauncher para elegir el perfil: LanLauncher si pasa `+exec <perfil>`, pero el servidor lo ignora y siempre
lee `storage/iw5/server.cfg` (lo comprobamos varias veces en el log). Por eso la app y los scripts copian el perfil sobre
`server.cfg` antes de arrancar.

Dentro de un bottle: "Ejecutar ejecutable" y elige `gestor-servidor.exe`. Alli busca `%LOCALAPPDATA%\Plutonium\storage\iw5`
de ese bottle y lanza el bootstrapper directamente.

Probado: abrir un perfil, editarlo, iniciar el servidor desde la app con `chico_ffa_nobots` (roto a `mp_nuked`, el primer mapa), y
detenerlo. El binario de Linux y el `.exe` (bajo Wine) arrancan. Sin probar: el `.exe` iniciando el servidor dentro de un bottle,
Bodega, y que el servidor rote de mapa al terminar una partida.

Recompilar si cambias el `.py` (PyInstaller; el resultado queda en `dist/`, copialo a esta carpeta):
```
python3 -m venv v && v/bin/pip install pyinstaller
v/bin/pyinstaller --onefile --windowed --name gestor-servidor-linux gestor_servidor.py
# Windows: lo mismo con Python para Windows (en Windows, o en un prefijo de Wine con Python instalado)
python -m PyInstaller --onefile --windowed --name gestor-servidor gestor_servidor.py
```
Compilado con Python 3.14 (Linux), 3.13.9 (Windows) y PyInstaller 6.x.

La version en Rust (eframe/egui) vive en `../gestor-rs/`.

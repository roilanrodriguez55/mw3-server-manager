# Gestor de servidor MW3 LAN en Rust (eframe/egui 0.37)

Reescritura de `../gestor-py/` con la misma funcion: cargar, editar y guardar perfiles `.cfg`, e iniciar o detener el servidor del
perfil seleccionado sin LanLauncher. Perfiles y reglas: `../perfiles/`. Scripts equivalentes sin ventana: `../scripts/`.

## Codigo
| Archivo | Contenido |
|---|---|
| `src/cfg.rs` | Formulario de un perfil: `parsear` (texto a formulario), `construir` (formulario a texto) y `validar`. Lo que no reconoce va a "lineas extra" |
| `src/ajustes.rs` | `gestor_servidor.json` junto al ejecutable, y deteccion de la carpeta `storage/iw5` (Bottles, Bodega, Wine, Lutris, Proton) y de wine |
| `src/servidor.rs` | Iniciar (copia el perfil sobre `server.cfg` con respaldo y lanza el bootstrapper), detener, comprobar puerto UDP y leer `console.log` |
| `src/app.rs`, `src/tema.rs` | Ventana (lista de perfiles, pestañas General, Rotacion, Bots y Texto, consola) y tema |

## Usar y compilar
```
cargo run                        # desarrollo
cargo test                       # 13 tests, incluido ida y vuelta de los perfiles de ../perfiles
# release con pocos recursos: 2 hilos, 3 GB de RAM como tope, prioridad baja
nice -n 10 systemd-run --user --scope -p MemoryMax=3G -p MemorySwapMax=0 -p CPUQuota=200% cargo build --release -j 2
# Windows (necesita zig, cargo-zigbuild y rustup target add x86_64-pc-windows-gnu)
nice -n 10 systemd-run --user --scope -p MemoryMax=3G -p MemorySwapMax=0 -p CPUQuota=200% cargo zigbuild --release --target x86_64-pc-windows-gnu -j 2
```
El perfil release usa `lto = "thin"` para gastar menos memoria. Tiempos medidos con esos limites en este equipo: Linux 8 min, Windows 10 min.
Los ejecutables salen en `target/release/gestor-servidor` y `target/x86_64-pc-windows-gnu/release/gestor-servidor.exe`
(17.8 MB y 11.7 MB); copiados a `bin/` como `gestor-servidor-linux` y `gestor-servidor.exe`.

## Estado y lo que no esta resuelto
- Probado: tests y clippy, y la interfaz abierta con `cargo run` (compilacion de desarrollo) por el propietario.
- Sin comprobar con la app real: iniciar y detener el servidor desde esta version, y que el servidor rote de mapa.
- En una prueba de arranque rapida con el binario release en una sesion Wayland, salio solo a los ~2.6 s con codigo 0. Con
  `WAYLAND_DISPLAY` quitado (X11) y con el binario de desarrollo se quedo abierto. No se encontro la causa.
- El `.exe` bajo el Wine del sistema fallo al abrir: `Wgpu(RequestDeviceError ... Device(Lost))`. eframe usa wgpu por defecto. Con
  el Wine del bottle (GE-Proton, DXVK) no se probo. Si falla tambien alli, la salida probable es compilar con el renderer `glow`
  (`eframe` con `default-features = false` y `features = ["glow", ...]`, y `renderer: eframe::Renderer::Glow` en `NativeOptions`).
- Por esas dos razones los ejecutables de `bin/` no estan en git hasta que se confirme que abren.
- Esc no cierra los dialogos, y el respaldo `server.cfg.bak-FECHA` usa hora UTC.

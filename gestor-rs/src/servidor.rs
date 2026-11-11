//! Arranque y parada del servidor Plutonium, comprobacion de puerto y lectura de console.log.
use crate::ajustes::{Ajustes, buscar_wine, dir_app};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[derive(Default)]
pub struct Servidor {
    pub hijo: Option<Child>,
    pub perfil: Option<String>, // nombre de archivo del perfil que corre
}

impl Servidor {
    /// true si el proceso iniciado por esta app sigue vivo (limpia `hijo` y `perfil` si termino).
    pub fn corriendo(&mut self) -> bool {
        let Some(hijo) = self.hijo.as_mut() else { return false };
        match hijo.try_wait() {
            Ok(None) => true,
            _ => {
                self.hijo = None;
                self.perfil = None;
                false
            }
        }
    }

    /// Copia `perfil` sobre storage/server.cfg (respaldo server.cfg.bak-YYYYmmdd-HHMMSS) y lanza el bootstrapper.
    pub fn iniciar(&mut self, aj: &Ajustes, perfil: &Path, puerto: &str) -> Result<(), String> {
        if self.corriendo() {
            return Err("Ya hay un servidor iniciado desde esta ventana. Detenlo primero.".into());
        }
        if !puerto_valido(puerto) {
            return Err(PUERTO_MAL.into());
        }
        let storage = PathBuf::from(&aj.storage);
        if aj.storage.is_empty() || !storage.is_dir() {
            return Err("Falta la carpeta storage/iw5 de Plutonium (Ajustes de rutas).".into());
        }
        let plu = plutonium_dir(&aj.storage).ok_or("Falta la carpeta storage/iw5 de Plutonium (Ajustes de rutas).")?;
        let exe = plu.join("bin").join("plutonium-bootstrapper-win32.exe");
        if !exe.exists() {
            return Err(format!("No existe {}", exe.display()));
        }

        let mut mw3 = aj.mw3.clone();
        if mw3.is_empty() {
            mw3 = fs::read_to_string(plu.join("config.json"))
                .ok()
                .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                .and_then(|v| v["iw5Path"].as_str().map(String::from))
                .ok_or("No pude leer iw5Path de config.json de Plutonium. Indica la carpeta de MW3 en Ajustes.")?;
        }
        if cfg!(not(windows)) && !tiene_unidad(&mw3) {
            mw3 = format!("Z:{}", mw3.replace('/', "\\"));
        }

        if !puerto_libre(puerto) {
            return Err(format!("El puerto UDP {puerto} ya esta en uso: hay otro servidor abierto. Cierralo o usa otro puerto."));
        }
        let sc = storage.join("server.cfg");
        if sc.exists() {
            fs::copy(&sc, storage.join(format!("server.cfg.bak-{}", marca_hora()))).map_err(|e| e.to_string())?;
        }
        fs::copy(perfil, &sc).map_err(|e| e.to_string())?;

        let args = ["iw5mp", mw3.as_str(), "-lan", "-dedicated", "+set", "net_port", puerto, "+map_rotate"];
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new(&exe);
            c.args(args);
            c
        } else {
            let wine = if aj.wine.is_empty() { buscar_wine() } else { aj.wine.clone() };
            if wine.is_empty() {
                return Err("Falta wine. Indicalo en Ajustes de rutas.".into());
            }
            let (prefijo, usuario) =
                prefijo_wine(&aj.storage).ok_or("No pude deducir el prefijo de Wine desde la ruta de storage.")?;
            let mut c = Command::new(&wine);
            c.arg(format!(r"C:\users\{usuario}\AppData\Local\Plutonium\bin\plutonium-bootstrapper-win32.exe"))
                .args(args)
                .env("WINEPREFIX", prefijo)
                .env("WINEDEBUG", "-all");
            #[cfg(unix)]
            c.process_group(0);
            c
        };
        // el bootstrapper busca games\iw5mp.exe relativo a la carpeta actual
        cmd.current_dir(&plu);
        let log = fs::File::create(dir_app().join("gestor_servidor.log"))
            .ok()
            .and_then(|f| f.try_clone().ok().map(|g| (f, g)));
        match log {
            Some((f, g)) => cmd.stdout(Stdio::from(f)).stderr(Stdio::from(g)),
            None => cmd.stdout(Stdio::null()).stderr(Stdio::null()),
        };

        let hijo = cmd.spawn().map_err(|e| e.to_string())?;
        self.hijo = Some(hijo);
        self.perfil = perfil.file_name().map(|n| n.to_string_lossy().into_owned());
        Ok(())
    }

    /// Detiene solo el proceso que inicio esta app.
    pub fn detener(&mut self) -> Result<(), String> {
        const NO_HAY: &str = "No hay un servidor iniciado desde esta ventana. Si lo abriste de otra forma, cierralo desde su ventana.";
        let pid = if self.corriendo() { self.hijo.as_ref().map(Child::id) } else { None };
        let Some(pid) = pid else { return Err(NO_HAY.into()) };
        #[cfg(unix)]
        {
            // SAFETY: killpg solo envia una senal al grupo de procesos del hijo.
            if unsafe { libc::killpg(pid as i32, libc::SIGTERM) } != 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
        }
        #[cfg(not(unix))]
        {
            let _ = pid;
            if let Some(hijo) = self.hijo.as_mut() {
                hijo.kill().map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}

pub const PUERTO_MAL: &str = "El puerto debe ser un numero entre 1 y 65535.";

/// true si el texto es un entero entre 1 y 65535.
pub fn puerto_valido(puerto: &str) -> bool {
    !puerto.is_empty() && puerto.bytes().all(|b| b.is_ascii_digit()) && puerto.parse::<u16>().is_ok_and(|p| p >= 1)
}

/// true si se puede abrir el puerto UDP.
pub fn puerto_libre(puerto: &str) -> bool {
    let Ok(p) = puerto.parse::<u16>() else { return false };
    std::net::UdpSocket::bind(("0.0.0.0", p)).is_ok()
}

/// Ultimas `lineas` lineas de <Plutonium>/console.log sin secuencias ANSI. Cadena vacia si no se puede leer.
pub fn leer_log(storage: &str, lineas: usize) -> String {
    let Some(plu) = plutonium_dir(storage) else { return String::new() };
    let Ok(mut f) = fs::File::open(plu.join("console.log")) else { return String::new() };
    let Ok(n) = f.seek(SeekFrom::End(0)) else { return String::new() };
    let mut buf = Vec::new();
    if f.seek(SeekFrom::Start(n.saturating_sub(12000))).is_err() || f.read_to_end(&mut buf).is_err() {
        return String::new();
    }
    let texto = sin_ansi(&String::from_utf8_lossy(&buf));
    let todas: Vec<&str> = texto.lines().collect();
    todas[todas.len().saturating_sub(lineas)..].join("\n")
}

/// <Plutonium> a partir de <Plutonium>/storage/iw5.
fn plutonium_dir(storage: &str) -> Option<PathBuf> {
    Path::new(storage).parent()?.parent().map(Path::to_path_buf)
}

/// true si la ruta empieza con letra de unidad y `:` (ej. `Z:\` o `C:/`).
fn tiene_unidad(ruta: &str) -> bool {
    let b = ruta.as_bytes();
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

/// (prefijo, usuario) de Wine desde `<prefijo>/drive_c/users/<usuario>/...`.
fn prefijo_wine(storage: &str) -> Option<(String, String)> {
    const DRIVE: &str = "/drive_c/users/";
    let i = storage.find(DRIVE)?;
    let usuario = storage[i + DRIVE.len()..].split('/').next().filter(|u| !u.is_empty())?;
    Some((storage[..i].to_string(), usuario.to_string()))
}

/// Quita secuencias ESC[...letra y ESC sueltos.
fn sin_ansi(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    let mut it = t.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\x1b' {
            out.push(c);
        } else if it.peek() == Some(&'[') {
            it.next();
            while it.peek().is_some_and(|d| d.is_ascii_digit() || *d == ';') {
                it.next();
            }
            if it.peek().is_some_and(|d| d.is_ascii_alphabetic()) {
                it.next();
            }
        }
    }
    out
}

// shortcut: UTC en vez de hora local, usar chrono si importa
fn marca_hora() -> String {
    let segundos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    marca_de(segundos as i64)
}

/// YYYYmmdd-HHMMSS (UTC) a partir de segundos Unix, con calendario gregoriano proleptico.
fn marca_de(segundos: i64) -> String {
    let (dias, resto) = (segundos.div_euclid(86400), segundos.rem_euclid(86400));
    let z = dias + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", resto / 3600, resto % 3600 / 60, resto % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;

    #[test]
    fn puerto_libre_detecta_socket_ocupado() {
        let s = UdpSocket::bind(("0.0.0.0", 0)).unwrap();
        let p = s.local_addr().unwrap().port().to_string();
        assert!(!puerto_libre(&p));
        drop(s);
        assert!(puerto_libre(&p));
        assert!(!puerto_libre("abc"));
    }

    #[test]
    fn puerto_valido_acota_entre_1_y_65535() {
        assert!(puerto_valido("1"));
        assert!(puerto_valido("65535"));
        assert!(!puerto_valido("0"));
        assert!(!puerto_valido("65536"));
        assert!(!puerto_valido(""));
        assert!(!puerto_valido("+80"));
    }

    #[test]
    fn leer_log_devuelve_ultimas_lineas_sin_ansi() {
        let dir = std::env::temp_dir().join(format!("gestor_servidor_test_log_{}", std::process::id()));
        let storage = dir.join("x").join("storage").join("iw5");
        fs::create_dir_all(&storage).unwrap();
        let texto: String = (1..=5).map(|i| format!("\x1b[38;2;255;255;255mhola {i}\x1b[0m\n")).collect();
        fs::write(dir.join("x").join("console.log"), texto).unwrap();
        assert_eq!(leer_log(storage.to_str().unwrap(), 2), "hola 4\nhola 5");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prefijo_wine_deduce_prefijo_y_usuario() {
        let s = "/home/u/.local/share/bottles/bottles/Games/drive_c/users/steamuser/AppData/Local/Plutonium/storage/iw5";
        assert_eq!(
            prefijo_wine(s),
            Some(("/home/u/.local/share/bottles/bottles/Games".into(), "steamuser".into()))
        );
        assert_eq!(prefijo_wine("/home/u/Juegos/Plutonium/storage/iw5"), None);
    }

    #[test]
    fn marca_de_formatea_utc() {
        assert_eq!(marca_de(1_700_000_000), "20231114-221320");
        assert_eq!(marca_de(0), "19700101-000000");
    }

    #[test]
    fn sin_ansi_quita_codigos() {
        assert_eq!(sin_ansi("\x1b[1;31mrojo\x1b[0m normal\x1b"), "rojo normal");
    }
}

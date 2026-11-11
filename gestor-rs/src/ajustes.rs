//! Ajustes guardados en gestor_servidor.json junto al ejecutable, y deteccion de rutas.
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Ajustes {
    pub perfiles: String, // carpeta con los mw3_*.cfg
    pub storage: String,  // .../Plutonium/storage/iw5
    pub mw3: String,      // carpeta de MW3 (vacio = la de config.json de Plutonium)
    pub wine: String,     // solo Linux
    pub puerto: String,
    pub ultimo: String, // ultimo perfil abierto (nombre de archivo)
}

/// Carpeta del ejecutable (o del cwd si no se puede saber).
pub fn dir_app() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| env::current_dir().unwrap_or_default())
}

fn archivo() -> PathBuf {
    dir_app().join("gestor_servidor.json")
}

/// Lee el JSON; si no existe o esta roto, devuelve valores por defecto (puerto "5000", carpeta de perfiles inicial).
pub fn cargar() -> Ajustes {
    let mut a: Ajustes = fs::read_to_string(archivo())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    if a.puerto.is_empty() {
        a.puerto = "5000".into();
    }
    if a.perfiles.is_empty() || !Path::new(&a.perfiles).is_dir() {
        a.perfiles = carpeta_inicial();
    }
    a
}

pub fn guardar(a: &Ajustes) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(a).map_err(std::io::Error::other)?;
    fs::write(archivo(), json)
}

/// Primera carpeta de perfiles con mw3_*.cfg entre el dir_app y sus 5 ancestros: primero `A/perfiles` en todos los ancestros, luego `A`.
/// Si no hay ninguna, crea `perfiles_guardados` junto al ejecutable.
fn carpeta_inicial() -> String {
    let base = dir_app();
    let ancestros: Vec<PathBuf> = base.ancestors().take(6).map(Path::to_path_buf).collect();
    let d = elegir_perfiles(&ancestros).unwrap_or_else(|| {
        let d = base.join("perfiles_guardados");
        let _ = fs::create_dir_all(&d);
        d
    });
    d.to_string_lossy().into_owned()
}

fn elegir_perfiles(ancestros: &[PathBuf]) -> Option<PathBuf> {
    ancestros
        .iter()
        .map(|a| a.join("perfiles"))
        .chain(ancestros.iter().cloned())
        .find(|d| tiene_perfiles(d))
}

fn tiene_perfiles(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|entradas| {
        entradas.flatten().any(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.starts_with("mw3_") && n.ends_with(".cfg")
        })
    })
}

/// Instalaciones de Plutonium (carpetas `.../Plutonium/storage/iw5`). En Windows solo %LOCALAPPDATA%.
pub fn buscar_storage() -> Vec<String> {
    if cfg!(windows) {
        let Ok(local) = env::var("LOCALAPPDATA") else {
            return vec![];
        };
        let p = PathBuf::from(local).join("Plutonium").join("storage").join("iw5");
        return if p.is_dir() { vec![p.to_string_lossy().into_owned()] } else { vec![] };
    }
    let Some(h) = env::var_os("HOME").map(PathBuf::from) else {
        return vec![];
    };
    let raices: Vec<PathBuf> = [
        ".var/app/com.usebottles.bottles/data/bottles",
        ".local/share/bottles",
        ".local/share/bodega",
        ".wine",
        "Games",
        ".local/share/lutris",
        ".local/share/Steam/steamapps/compatdata",
    ]
    .iter()
    .map(|r| h.join(r))
    .filter(|p| p.is_dir())
    .collect();
    if raices.is_empty() {
        return vec![];
    }
    Command::new("find")
        .args(&raices)
        .args(["-maxdepth", "12", "-type", "d", "-path", "*/Plutonium/storage/iw5"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(str::to_string).collect())
        .unwrap_or_default()
}

/// Ejecutables `relativo` dentro de las subcarpetas de `base` que empiezan con `prefijo`, de mayor a menor nombre.
fn wines_en(base: &Path, prefijo: &str, relativo: &str) -> Vec<PathBuf> {
    let Ok(entradas) = fs::read_dir(base) else {
        return vec![];
    };
    let mut nombres: Vec<String> = entradas
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(prefijo))
        .collect();
    nombres.sort_by(|a, b| b.cmp(a));
    nombres
        .into_iter()
        .map(|n| base.join(n).join(relativo))
        .filter(|p| p.is_file())
        .collect()
}

/// Wine del bottle (GE-Proton de Steam, runtimes de Bodega) o el del PATH. Cadena vacia si no hay.
pub fn buscar_wine() -> String {
    if cfg!(windows) {
        return String::new();
    }
    let mut cand = Vec::new();
    if let Some(h) = env::var_os("HOME").map(PathBuf::from) {
        cand.extend(wines_en(&h.join(".local/share/Steam/compatibilitytools.d"), "GE-Proton", "files/bin/wine"));
        cand.extend(wines_en(&h.join(".local/share/bodega/runtimes"), "wine-", "bin/wine"));
    }
    if let Some(p) = env::var_os("PATH") {
        cand.extend(env::split_paths(&p).map(|d| d.join("wine")));
    }
    cand.into_iter()
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ida_y_vuelta_serde() {
        let a = Ajustes {
            perfiles: "/p".into(),
            storage: "/s".into(),
            mw3: "/m".into(),
            wine: "/w".into(),
            puerto: "6000".into(),
            ultimo: "mw3_x.cfg".into(),
        };
        let json = serde_json::to_string_pretty(&a).unwrap();
        let b: Ajustes = serde_json::from_str(&json).unwrap();
        assert_eq!(b.perfiles, "/p");
        assert_eq!(b.storage, "/s");
        assert_eq!(b.mw3, "/m");
        assert_eq!(b.wine, "/w");
        assert_eq!(b.puerto, "6000");
        assert_eq!(b.ultimo, "mw3_x.cfg");
    }

    #[test]
    fn campos_faltantes_quedan_vacios() {
        let b: Ajustes = serde_json::from_str(r#"{"puerto":"6000"}"#).unwrap();
        assert_eq!(b.puerto, "6000");
        assert!(b.perfiles.is_empty());
        assert!(b.storage.is_empty());
        assert!(b.mw3.is_empty());
        assert!(b.wine.is_empty());
        assert!(b.ultimo.is_empty());
    }

    #[test]
    fn elegir_perfiles_prefiere_subcarpeta_perfiles_de_cualquier_ancestro() {
        let t = env::temp_dir().join(format!("gestor_ajustes_elegir_{}", std::process::id()));
        let (cerca, medio) = (t.join("a").join("b"), t.join("a"));
        fs::create_dir_all(&cerca).unwrap();
        fs::create_dir_all(t.join("perfiles")).unwrap();
        fs::write(cerca.join("mw3_cerca.cfg"), "").unwrap();
        fs::write(t.join("perfiles").join("mw3_x.cfg"), "").unwrap();
        let ancestros = vec![cerca.clone(), medio.clone(), t.clone()];
        assert_eq!(elegir_perfiles(&ancestros), Some(t.join("perfiles")));

        fs::remove_file(t.join("perfiles").join("mw3_x.cfg")).unwrap();
        assert_eq!(elegir_perfiles(&ancestros), Some(cerca.clone()));

        fs::remove_file(cerca.join("mw3_cerca.cfg")).unwrap();
        assert_eq!(elegir_perfiles(&ancestros), None);
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    fn tiene_perfiles_detecta_cfg() {
        let dir = env::temp_dir().join(format!("gestor_ajustes_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert!(!tiene_perfiles(&dir));
        fs::write(dir.join("otro.txt"), "").unwrap();
        assert!(!tiene_perfiles(&dir));
        fs::write(dir.join("mw3_x.cfg"), "").unwrap();
        assert!(tiene_perfiles(&dir));
        fs::remove_dir_all(&dir).unwrap();
    }
}

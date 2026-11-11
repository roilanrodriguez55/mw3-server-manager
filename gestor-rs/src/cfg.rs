//! Formulario de un perfil .cfg: parsear texto -> Form y construir Form -> texto.

pub const MODOS: &[(&str, &str)] = &[
    ("war", "Team Deathmatch"), ("dm", "Todos contra todos"), ("dom", "Dominacion"), ("sd", "Buscar y Destruir"),
    ("sab", "Sabotaje"), ("koth", "HQ"), ("ctf", "Captura la bandera"), ("tdef", "Team Defender"),
    ("dd", "Demolicion"), ("conf", "Kill Confirmed"), ("grnd", "Drop Zone"), ("gun", "Gun Game"),
    ("infect", "Infectados"), ("oic", "Una en la recamara"), ("jugg", "Juggernaut"), ("tjugg", "Team Juggernaut"),
];
pub const BASE: &[(&str, &str)] = &[
    ("mp_alpha", "Lockdown"), ("mp_bootleg", "Bootleg"), ("mp_bravo", "Mission"), ("mp_carbon", "Carbon"),
    ("mp_dome", "Dome"), ("mp_exchange", "Downturn"), ("mp_hardhat", "Hardhat"), ("mp_interchange", "Interchange"),
    ("mp_lambeth", "Fallen"), ("mp_mogadishu", "Bakaara"), ("mp_paris", "Resistance"), ("mp_plaza2", "Arkaden"),
    ("mp_radar", "Outpost"), ("mp_seatown", "Seatown"), ("mp_underground", "Underground"), ("mp_village", "Village"),
];
pub const EXTRA: &[(&str, &str)] = &[
    ("mp_nuked", "Nuketown (Plutonium)"), ("mp_rust", "Rust (Plutonium)"), ("mp_nightshift", "Skidrow (Plutonium)"),
    ("mp_favela", "Favela (Plutonium)"), ("mp_highrise", "Highrise (Plutonium)"),
];
/// Modos por rondas: solo ellos llevan roundlimit y winlimit.
pub const POR_RONDAS: &[&str] = &["sd", "sab", "dd"];
/// Comportamientos bots_play_*, en el orden de `Form::play`.
pub const PLAY: [&str; 12] = ["move", "knife", "fire", "nade", "take_carepackages", "obj", "camp", "jumpdrop", "target_other", "killstreak", "ads", "aim"];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Bots { Ninguno, #[default] Fill, Add }

#[derive(Clone, PartialEq, Debug)]
pub struct Form {
    pub nombre: String, // nombre del archivo sin mw3_ ni .cfg (no sale en el texto)
    pub hostname: String, pub maxclients: String, pub password: String, pub rcon: String,
    pub privados: String, pub privpass: String, pub voz: String, pub votos: String,
    pub chat_muertos: String, pub inactividad: String, pub ban: String, pub flood: String,
    pub fuego_amigo: String, pub rachas: String, pub respawn_forzado: String,
    pub tiempo: String, pub puntos: String, pub rondas: String, pub ganar: String, pub vidas: String,
    pub bots: Bots, pub bots_n: String, pub fillmode: String, pub kick: bool, pub spec: bool, pub watch: bool,
    pub skill: String, pub skmin: String, pub skmax: String, pub equipo: String, pub team_n: String, pub team_mode: String,
    pub play: [bool; 12],
    pub rotacion: Vec<(String, String)>, // (modo, mapa)
    pub extras: String,                  // lineas que el formulario no conoce, una por linea
}

impl Default for Form {
    fn default() -> Self {
        Form {
            nombre: "mi_perfil".into(),
            hostname: "MW3 LAN".into(), maxclients: "16".into(), password: String::new(), rcon: String::new(),
            privados: "0".into(), privpass: String::new(), voz: "2".into(), votos: "1".into(),
            chat_muertos: "0".into(), inactividad: "90".into(), ban: "300".into(), flood: "1".into(),
            fuego_amigo: String::new(), rachas: String::new(), respawn_forzado: String::new(),
            tiempo: "10".into(), puntos: String::new(), rondas: String::new(), ganar: String::new(), vidas: String::new(),
            bots: Bots::Fill, bots_n: "10".into(), fillmode: "0".into(), kick: true, spec: true, watch: false,
            skill: "0".into(), skmin: "1".into(), skmax: "7".into(), equipo: "autoassign".into(), team_n: "0".into(), team_mode: "0".into(),
            play: [true; 12],
            rotacion: Vec::new(),
            extras: String::new(),
        }
    }
}

/// Texto de un .cfg -> formulario. Lo que no reconoce va a `extras`. `nombre` queda vacio.
pub fn parsear(texto: &str) -> Form {
    let mut f = Form { nombre: String::new(), ..Form::default() };
    let mut extras: Vec<&str> = Vec::new();
    let mut scr: Vec<(String, String, &str, &str)> = Vec::new(); // (modo, limite, valor, linea)
    let mut add: Option<&str> = None;
    let mut fill: Option<&str> = None;
    let mut rotacion: Vec<(String, String)> = Vec::new();
    for linea in texto.lines() {
        let s = linea.trim();
        if s.is_empty() || s.starts_with("//") { continue; }
        let Some((clave, val)) = asignacion(s) else { extras.push(s); continue; };
        let k = clave.to_ascii_lowercase();
        match k.as_str() {
            "sv_maprotation" => match mapa_rotacion(val) { Some(r) => rotacion = r, None => extras.push(s) },
            "g_gametype" => {}
            "bots_manage_add" => add = Some(val),
            "bots_manage_fill" => fill = Some(val),
            "bots_manage_fill_mode" => f.fillmode = val.chars().take(1).collect(),
            "bots_manage_fill_kick" => f.kick = val == "1",
            "bots_manage_fill_spec" => f.spec = val == "1",
            "bots_manage_fill_watchplayers" => f.watch = val == "1",
            "sv_hostname" => f.hostname = val.into(),
            "sv_maxclients" => f.maxclients = val.into(),
            "g_password" => f.password = val.into(),
            "rcon_password" => f.rcon = val.into(),
            "sv_privateclients" => f.privados = val.into(),
            "sv_privatepassword" => f.privpass = val.into(),
            "sv_voice" => f.voz = val.into(),
            "g_allowvote" => f.votos = val.into(),
            "g_deadchat" => f.chat_muertos = val.into(),
            "g_inactivity" => f.inactividad = val.into(),
            "sv_kickbantime" => f.ban = val.into(),
            "sv_floodprotect" => f.flood = val.into(),
            "scr_team_fftype" => f.fuego_amigo = val.into(),
            "scr_game_hardpoints" => f.rachas = val.into(),
            "scr_player_forcerespawn" => f.respawn_forzado = val.into(),
            "bots_skill" => f.skill = val.into(),
            "bots_skill_min" => f.skmin = val.into(),
            "bots_skill_max" => f.skmax = val.into(),
            "bots_team" => f.equipo = val.into(),
            "bots_team_amount" => f.team_n = val.into(),
            "bots_team_mode" => f.team_mode = val.into(),
            _ if k.starts_with("bots_play_") => match PLAY.iter().position(|p| *p == &k[10..]) {
                Some(i) => f.play[i] = val != "0",
                None => extras.push(s),
            },
            _ => match clave_scr(&k) {
                Some((modo, d)) => scr.push((modo, d, val, s)),
                None => extras.push(s),
            },
        }
    }
    let activo = |s: &&str| !s.is_empty() && *s != "0";
    if let Some(n) = fill.filter(activo) { f.bots = Bots::Fill; f.bots_n = n.into(); }
    else if let Some(n) = add.filter(activo) { f.bots = Bots::Add; f.bots_n = n.into(); }
    else { f.bots = Bots::Ninguno; }

    // Los limites salen del texto solo si la linea es la primera de su tipo; las demas van a extras.
    let modos: Vec<&str> = rotacion.iter().map(|(m, _)| m.as_str()).collect();
    let mut primero: Vec<(&str, &str)> = Vec::new();
    for (_, d, val, _) in &scr {
        if !primero.iter().any(|(x, _)| *x == d.as_str()) { primero.push((d.as_str(), *val)); }
    }
    for (modo, d, val, linea) in &scr {
        let primer = primero.iter().find(|(x, _)| *x == d.as_str()).map(|(_, v)| *v);
        if !(modos.contains(&modo.as_str()) && primer == Some(*val)) { extras.push(*linea); }
    }
    f.tiempo.clear(); f.puntos.clear(); f.rondas.clear(); f.ganar.clear(); f.vidas.clear();
    for (d, val) in primero {
        let campo = match d {
            "timelimit" => &mut f.tiempo,
            "scorelimit" => &mut f.puntos,
            "roundlimit" => &mut f.rondas,
            "winlimit" => &mut f.ganar,
            "numlives" => &mut f.vidas,
            _ => continue,
        };
        *campo = val.into();
    }
    f.rotacion = rotacion;
    f.extras = extras.join("\n");
    f
}

/// "set|seta <clave> <valor> [// comentario]" -> (clave, valor sin comillas externas).
fn asignacion(s: &str) -> Option<(&str, &str)> {
    let (cmd, resto) = s.split_once(char::is_whitespace)?;
    if !cmd.eq_ignore_ascii_case("set") && !cmd.eq_ignore_ascii_case("seta") { return None; }
    let (clave, resto) = resto.trim_start().split_once(char::is_whitespace)?;
    let val = resto.trim_start();
    let val = val.find("//").map_or(val, |i| &val[..i]).trim_end();
    let val = if val.len() > 1 && val.starts_with('"') && val.ends_with('"') { &val[1..val.len() - 1] } else { val };
    Some((clave, val))
}

/// sv_maprotation "gametype X map Y ..." -> [(X, Y)]. None si hay un token que no encaja.
fn mapa_rotacion(val: &str) -> Option<Vec<(String, String)>> {
    let mut ent = Vec::new();
    let mut modo: Option<&str> = None;
    let mut t = val.split_whitespace();
    while let Some(tok) = t.next() {
        let arg = t.next()?;
        match tok {
            "gametype" => modo = Some(arg),
            "map" => ent.push((modo?.to_string(), arg.to_string())),
            _ => return None,
        }
    }
    Some(ent)
}

/// scr_<modo>_<limite> con modo en minusculas y limite de los cinco de LIMITES.
fn clave_scr(k: &str) -> Option<(String, String)> {
    let (modo, d) = k.strip_prefix("scr_")?.split_once('_')?;
    let limite = ["timelimit", "scorelimit", "roundlimit", "winlimit", "numlives"].contains(&d);
    (!modo.is_empty() && modo.bytes().all(|b| b.is_ascii_lowercase()) && limite).then(|| (modo.to_string(), d.to_string()))
}

/// Formulario -> texto del .cfg.
pub fn construir(f: &Form) -> String {
    let nombre = if f.nombre.is_empty() { "perfil" } else { f.nombre.as_str() };
    let mut l: Vec<String> = vec![format!("// {nombre}: generado por gestor_servidor.py")];
    let simples: [(&String, &str, bool); 15] = [
        (&f.hostname, "sv_hostname", true), (&f.maxclients, "sv_maxclients", false), (&f.password, "g_password", true),
        (&f.rcon, "rcon_password", true), (&f.privados, "sv_privateClients", false), (&f.privpass, "sv_privatePassword", true),
        (&f.voz, "sv_voice", false), (&f.votos, "g_allowVote", false), (&f.chat_muertos, "g_deadChat", false),
        (&f.inactividad, "g_inactivity", false), (&f.ban, "sv_kickBanTime", false), (&f.flood, "sv_floodProtect", false),
        (&f.fuego_amigo, "scr_team_fftype", false), (&f.rachas, "scr_game_hardpoints", false),
        (&f.respawn_forzado, "scr_player_forcerespawn", false),
    ];
    for (val, dvar, texto) in simples {
        if texto { l.push(format!("set {dvar} \"{}\"", val.replace('"', ""))); }
        else if !val.is_empty() { l.push(format!("set {dvar} {val}")); }
    }

    let mut rot: Vec<String> = Vec::new();
    let mut actual: Option<&String> = None;
    for (modo, mapa) in &f.rotacion {
        if actual != Some(modo) { rot.push(format!("gametype {modo}")); actual = Some(modo); }
        rot.push(format!("map {mapa}"));
    }
    let mut modos: Vec<&str> = Vec::new();
    for (m, _) in &f.rotacion { if !modos.contains(&m.as_str()) { modos.push(m.as_str()); } }
    if !rot.is_empty() {
        l.push(format!("set sv_maprotation \"{}\"", rot.join(" ")));
        l.push(format!("set g_gametype {}", modos[0]));
    }

    let limites = [(&f.tiempo, "timelimit", false), (&f.puntos, "scorelimit", false), (&f.rondas, "roundlimit", true),
                   (&f.ganar, "winlimit", true), (&f.vidas, "numlives", false)];
    let mut lim: Vec<String> = Vec::new();
    for m in &modos {
        for (val, d, solo_rondas) in &limites {
            if !val.is_empty() && (!*solo_rondas || POR_RONDAS.contains(m)) { lim.push(format!("set scr_{m}_{d} {val}")); }
        }
    }
    if !lim.is_empty() { l.push(String::new()); l.push("// Duracion y puntuacion".to_string()); l.extend(lim); }

    l.push(String::new());
    l.push("// Bot Warfare".to_string());
    match f.bots {
        Bots::Ninguno => {
            l.push("set bots_manage_add 0".to_string());
            l.push("set bots_manage_fill 0".to_string());
        }
        Bots::Fill => {
            l.push("set bots_manage_add 0".to_string());
            l.push(format!("set bots_manage_fill {}", f.bots_n));
            l.push(format!("set bots_manage_fill_mode {}", f.fillmode.chars().take(1).collect::<String>()));
            l.push(format!("set bots_manage_fill_kick {}", u8::from(f.kick)));
            l.push(format!("set bots_manage_fill_spec {}", u8::from(f.spec)));
            l.push(format!("set bots_manage_fill_watchplayers {}", u8::from(f.watch)));
        }
        Bots::Add => {
            l.push(format!("set bots_manage_add {}", f.bots_n));
            l.push("set bots_manage_fill 0".to_string());
        }
    }
    if f.bots != Bots::Ninguno {
        l.push(format!("set bots_skill {}", f.skill));
        l.push(format!("set bots_skill_min {}", f.skmin));
        l.push(format!("set bots_skill_max {}", f.skmax));
        l.push(format!("set bots_team {}", f.equipo));
        l.push(format!("set bots_team_amount {}", f.team_n));
        l.push(format!("set bots_team_mode {}", f.team_mode));
        for (i, p) in PLAY.iter().enumerate() {
            if !f.play[i] { l.push(format!("set bots_play_{p} 0")); }
        }
    }

    let extras = f.extras.trim();
    if !extras.is_empty() { l.push(String::new()); l.push("// Lineas extra".to_string()); l.extend(extras.lines().map(String::from)); }
    l.join("\n") + "\n"
}

/// Mensaje de error si el formulario no se puede guardar (nombre, rotacion, jugadores 1-18, numeros).
pub fn validar(f: &Form) -> Result<(), String> {
    if f.nombre.is_empty() || !f.nombre.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err("El nombre del archivo solo admite letras, numeros, _ y -.".into());
    }
    if f.rotacion.is_empty() { return Err("Agrega al menos un mapa a la rotacion.".into()); }
    if !(digitos(&f.maxclients) && f.maxclients.parse::<u32>().is_ok_and(|n| (1..=18).contains(&n))) {
        return Err("Maximo de jugadores: de 1 a 18.".into());
    }
    for (nombre, val) in [("tiempo", &f.tiempo), ("puntos", &f.puntos), ("rondas", &f.rondas), ("ganar", &f.ganar), ("vidas", &f.vidas)] {
        if !val.is_empty() && !numero(val) { return Err(format!("'{nombre}' debe ser un numero o estar vacio.")); }
    }
    if f.bots != Bots::Ninguno && !digitos(&f.bots_n) { return Err("N de bots debe ser un numero.".into()); }
    Ok(())
}

fn digitos(s: &str) -> bool { !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) }
/// Entero o decimal con punto, como `\d+(\.\d+)?`.
fn numero(s: &str) -> bool {
    match s.split_once('.') { Some((a, b)) => digitos(a) && digitos(b), None => digitos(s) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pares (clave en minusculas, valor) de las lineas set/seta de un texto.
    fn pares(texto: &str) -> Vec<(String, String)> {
        texto.lines().filter_map(|l| asignacion(l.trim()).map(|(k, v)| (k.to_ascii_lowercase(), v.to_string()))).collect()
    }

    #[test]
    fn valores_por_defecto() {
        let f = Form::default();
        assert_eq!(f.nombre, "mi_perfil");
        assert_eq!((f.hostname.as_str(), f.maxclients.as_str(), f.tiempo.as_str()), ("MW3 LAN", "16", "10"));
        assert_eq!((f.bots, f.bots_n.as_str(), f.kick, f.spec, f.watch), (Bots::Fill, "10", true, true, false));
        assert_eq!((f.equipo.as_str(), f.skmax.as_str()), ("autoassign", "7"));
        assert!(f.play.iter().all(|&p| p));
        assert!(f.rotacion.is_empty() && f.extras.is_empty());
    }

    #[test]
    fn validar_casos() {
        let mut f = Form { nombre: "tdm_prueba".into(), rotacion: vec![("war".into(), "mp_dome".into())], ..Form::default() };
        assert_eq!(validar(&f), Ok(()));
        f.maxclients = "19".into();
        assert!(validar(&f).is_err());
        f.maxclients = "16".into();
        f.puntos = "1.5.2".into();
        assert!(validar(&f).is_err());
    }

    #[test]
    fn perfiles_ida_y_vuelta() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../perfiles");
        let mut n = 0;
        for entrada in std::fs::read_dir(dir).unwrap() {
            let ruta = entrada.unwrap().path();
            let nombre = ruta.file_name().unwrap().to_string_lossy().into_owned();
            if !(nombre.starts_with("mw3_") && nombre.ends_with(".cfg")) { continue; }
            let texto = std::fs::read_to_string(&ruta).unwrap();
            let f1 = parsear(&texto);
            let t2 = construir(&f1);
            let f2 = parsear(&t2);
            assert_eq!(f1, f2, "{nombre}");
            let salida = pares(&t2);
            for par in pares(&texto) {
                assert!(salida.contains(&par), "{nombre}: falta {par:?} en la salida");
            }
            n += 1;
        }
        assert!(n > 0);
    }
}

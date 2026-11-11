#!/usr/bin/env python3
"""Gestor de servidores LAN de MW3 (Plutonium IW5) con Bot Warfare.
Carga, edita y guarda perfiles .cfg, y inicia/detiene el servidor del perfil seleccionado, sin LanLauncher.
Uso: python3 gestor_servidor.py   (Linux: Wine; Windows o dentro de un bottle: ejecutable directo)"""
import glob, json, os, re, shutil, signal, socket, subprocess, sys, time, tkinter as tk
from tkinter import ttk, messagebox, filedialog, simpledialog

AQUI = os.path.dirname(os.path.abspath(sys.executable if getattr(sys, "frozen", False) else __file__))
AJUSTES = os.path.join(AQUI, "gestor_servidor.json")
MODOS = {"war": "Team Deathmatch", "dm": "Todos contra todos", "dom": "Dominacion", "sd": "Buscar y Destruir",
         "sab": "Sabotaje", "koth": "HQ", "ctf": "Captura la bandera", "tdef": "Team Defender",
         "dd": "Demolicion", "conf": "Kill Confirmed", "grnd": "Drop Zone", "gun": "Gun Game",
         "infect": "Infectados", "oic": "Una en la recamara", "jugg": "Juggernaut", "tjugg": "Team Juggernaut"}
BASE = {"mp_alpha": "Lockdown", "mp_bootleg": "Bootleg", "mp_bravo": "Mission", "mp_carbon": "Carbon",
        "mp_dome": "Dome", "mp_exchange": "Downturn", "mp_hardhat": "Hardhat", "mp_interchange": "Interchange",
        "mp_lambeth": "Fallen", "mp_mogadishu": "Bakaara", "mp_paris": "Resistance", "mp_plaza2": "Arkaden",
        "mp_radar": "Outpost", "mp_seatown": "Seatown", "mp_underground": "Underground", "mp_village": "Village"}
EXTRA = {"mp_nuked": "Nuketown (Plutonium)", "mp_rust": "Rust (Plutonium)", "mp_nightshift": "Skidrow (Plutonium)",
         "mp_favela": "Favela (Plutonium)", "mp_highrise": "Highrise (Plutonium)"}
MAPAS = {**BASE, **EXTRA}
POR_RONDAS = ("sd", "sab", "dd")
SKILLS = ["0 aleatorio (entre min y max)", "1 facil", "2", "3", "4", "5", "6", "7 dificil", "8 personalizada", "9 totalmente aleatorio"]
EQUIPOS = ["autoassign", "allies", "axis", "spectator"]
FILL_MODOS = ["0 cuenta a todos", "1 solo bots", "2 por equipo (mapas)", "3 por equipo con bots"]
PLAY = ["move", "knife", "fire", "nade", "take_carepackages", "obj", "camp", "jumpdrop", "target_other", "killstreak", "ads", "aim"]

# variable del formulario -> dvar (se escribe solo si no esta vacia)
SIMPLES = dict(hostname="sv_hostname", maxclients="sv_maxclients", password="g_password", rcon="rcon_password",
               privados="sv_privateClients", privpass="sv_privatePassword", voz="sv_voice", votos="g_allowVote",
               chat_muertos="g_deadChat", inactividad="g_inactivity", ban="sv_kickBanTime", flood="sv_floodProtect",
               fuego_amigo="scr_team_fftype", rachas="scr_game_hardpoints", respawn_forzado="scr_player_forcerespawn")
TEXTO = ("hostname", "password", "rcon", "privpass")  # van entre comillas
LIMITES = dict(tiempo="timelimit", puntos="scorelimit", rondas="roundlimit", ganar="winlimit", vidas="numlives")
BOTVARS = dict(bots_n=None, kick="bots_manage_fill_kick", spec="bots_manage_fill_spec", watch="bots_manage_fill_watchplayers",
               skill="bots_skill", skmin="bots_skill_min", skmax="bots_skill_max", equipo="bots_team",
               team_n="bots_team_amount", team_mode="bots_team_mode")
DEFECTO = dict(hostname="MW3 LAN", maxclients="16", password="", rcon="", privados="0", privpass="", voz="2", votos="1",
               chat_muertos="0", inactividad="90", ban="300", flood="1", fuego_amigo="", rachas="", respawn_forzado="",
               tiempo="10", puntos="", rondas="", ganar="", vidas="", bots="fill", bots_n="10", fillmode="0", kick="1",
               spec="1", watch="0", skill="0", skmin="1", skmax="7", equipo="autoassign", team_n="0", team_mode="0")


def nombre(d, k): return d.get(k, k)
def etiqueta(d, k): return f"{nombre(d, k)} ({k})"
def codigo(t): return t.rsplit("(", 1)[-1].rstrip(")") if "(" in t else t.strip()
def sin_ansi(t): return re.sub(r"\x1b\[[0-9;]*[A-Za-z]|\x1b", "", t)


# ---------- cfg <-> formulario ----------
def construir(c):
    """c: valores del formulario (dict de str) + 'rotacion' [(modo, mapa)] + 'play' {nombre: '0'|'1'} + 'extras' str."""
    rot, actual = [], None
    for modo, mapa in c["rotacion"]:
        if modo != actual: rot.append(f"gametype {modo}"); actual = modo
        rot.append(f"map {mapa}")
    modos = list(dict.fromkeys(m for m, _ in c["rotacion"]))
    L = [f'// {c.get("nombre", "perfil")}: generado por gestor_servidor.py']
    for v, dvar in SIMPLES.items():
        if c[v] != "" or v in TEXTO:  # contrasenas siempre explicitas, aunque vacias
            L.append(f'set {dvar} "{c[v]}"' if v in TEXTO else f'set {dvar} {c[v]}')
    if rot: L += [f'set sv_maprotation "{" ".join(rot)}"', f"set g_gametype {modos[0]}"]
    lim = [f"set scr_{m}_{d} {c[v]}" for m in modos for v, d in LIMITES.items()
           if c[v] != "" and (v not in ("rondas", "ganar") or m in POR_RONDAS)]
    if lim: L += ["", "// Duracion y puntuacion"] + lim
    L += ["", "// Bot Warfare"]
    b = c["bots"]
    if b == "ninguno": L += ["set bots_manage_add 0", "set bots_manage_fill 0"]
    else:
        if b == "fill":
            L += ["set bots_manage_add 0", f'set bots_manage_fill {c["bots_n"]}', f'set bots_manage_fill_mode {c["fillmode"]}',
                  f'set bots_manage_fill_kick {c["kick"]}', f'set bots_manage_fill_spec {c["spec"]}',
                  f'set bots_manage_fill_watchplayers {c["watch"]}']
        else: L += [f'set bots_manage_add {c["bots_n"]}', "set bots_manage_fill 0"]
        L += [f'set bots_skill {c["skill"]}', f'set bots_skill_min {c["skmin"]}', f'set bots_skill_max {c["skmax"]}',
              f'set bots_team {c["equipo"]}', f'set bots_team_amount {c["team_n"]}', f'set bots_team_mode {c["team_mode"]}']
        L += [f"set bots_play_{p} {v}" for p, v in c["play"].items() if v == "0"]
    if c["extras"].strip(): L += ["", "// Lineas extra"] + c["extras"].strip().splitlines()
    return "\n".join(L) + "\n"


def parsear(texto):
    """Devuelve (valores, rotacion, play, extras) a partir del texto de un cfg. Lo que no reconoce va a extras."""
    v, play, extras, scr, rotacion = dict(DEFECTO), {p: "1" for p in PLAY}, [], [], []
    inv = {d.lower(): k for k, d in SIMPLES.items()}
    inv.update({d.lower(): k for k, d in BOTVARS.items() if d})
    add = fill = None
    for linea in texto.splitlines():
        s = linea.strip()
        if not s or s.startswith("//"): continue
        m = re.match(r"^seta?\s+(\S+)\s+(.*?)\s*(?://.*)?$", s, re.I)
        if not m: extras.append(s); continue
        k, val = m.group(1).lower(), m.group(2).strip()
        val = val[1:-1] if len(val) > 1 and val[0] == val[-1] == '"' else val
        if k == "sv_maprotation":
            t, modo, ok = val.split(), None, True
            ent = []
            i = 0
            while i < len(t):
                if t[i] == "gametype" and i + 1 < len(t): modo = t[i + 1]; i += 2
                elif t[i] == "map" and i + 1 < len(t) and modo: ent.append((modo, t[i + 1])); i += 2
                else: ok = False; break
            if ok: rotacion = ent
            else: extras.append(s)
        elif k == "g_gametype": pass
        elif k == "bots_manage_add": add = val
        elif k == "bots_manage_fill": fill = val
        elif k == "bots_manage_fill_mode": v["fillmode"] = val[:1]
        elif k.startswith("bots_play_") and k[10:] in PLAY: play[k[10:]] = val
        elif k in inv: v[inv[k]] = val
        elif re.match(r"^scr_[a-z]+_(timelimit|scorelimit|roundlimit|winlimit|numlives)$", k): scr.append((k, val, s))
        else: extras.append(s)
    v["bots"] = "fill" if fill not in (None, "0", "") else ("add" if add not in (None, "0", "") else "ninguno")
    if v["bots"] != "ninguno": v["bots_n"] = fill if v["bots"] == "fill" else add
    for var in LIMITES: v[var] = ""
    vistos = {}
    modos = {m for m, _ in rotacion}
    for k, val, s in scr:
        _, modo, d = k.split("_", 2)
        var = next(x for x, y in LIMITES.items() if y == d)
        vistos.setdefault(var, val)
        if modo in modos and val == vistos[var]: v[var] = val
        else: extras.append(s)
    for var in vistos: v[var] = vistos[var]
    return v, rotacion, play, "\n".join(extras)


# ---------- servidor ----------
def buscar_storage():
    if os.name == "nt":
        p = os.path.expandvars(r"%LOCALAPPDATA%\Plutonium\storage\iw5")
        return [p] if os.path.isdir(p) else []
    h = os.path.expanduser("~")
    raices = [f"{h}/.var/app/com.usebottles.bottles/data/bottles", f"{h}/.local/share/bottles", f"{h}/.local/share/bodega",
              f"{h}/.wine", f"{h}/Games", f"{h}/.local/share/lutris", f"{h}/.local/share/Steam/steamapps/compatdata"]
    raices = [r for r in raices if os.path.isdir(r)]
    if not raices: return []
    try:
        return subprocess.run(["find", *raices, "-maxdepth", "12", "-type", "d", "-path", "*/Plutonium/storage/iw5"],
                              capture_output=True, text=True, timeout=60).stdout.split()
    except Exception: return []


def buscar_wine():
    h = os.path.expanduser("~")
    for patron in (f"{h}/.local/share/Steam/compatibilitytools.d/GE-Proton*/files/bin/wine",
                   f"{h}/.local/share/bodega/runtimes/wine-*/bin/wine"):
        for w in sorted(glob.glob(patron), reverse=True):
            if os.access(w, os.X_OK): return w
    return shutil.which("wine") or ""


def puerto_libre(p):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try: s.bind(("0.0.0.0", int(p))); return True
    except OSError: return False
    finally: s.close()


class App(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Gestor de servidor MW3 LAN")
        self.geometry("1240x800")
        self.aj = dict(perfiles="", storage="", mw3="", wine="", puerto="5000", ultimo="")
        try: self.aj.update(json.load(open(AJUSTES)))
        except Exception: pass
        if not self.aj["perfiles"] or not os.path.isdir(self.aj["perfiles"]): self.aj["perfiles"] = self.carpeta_inicial()
        self.v = {k: tk.StringVar(value=d) for k, d in DEFECTO.items()}
        self.v["nombre"] = tk.StringVar(value="mi_perfil")
        self.play = {p: tk.StringVar(value="1") for p in PLAY}
        self.rotacion, self.archivo, self.manual, self.cargando = [], None, False, True
        self.proc, self.perfil_activo, self._sucio = None, None, False
        self._ui()
        self.cargando = False
        for v in [*self.v.values(), *self.play.values()]: v.trace_add("write", lambda *_: self.cambio())
        self.extras.bind("<<Modified>>", self._mod_extras)
        self.listar()
        if self.aj["ultimo"] and os.path.exists(os.path.join(self.aj["perfiles"], self.aj["ultimo"])):
            self.abrir(os.path.join(self.aj["perfiles"], self.aj["ultimo"]))
        else: self.regenerar(); self.sucio(False)
        self.after(1000, self.tic)
        self.after(300, self.autodetectar)
        self.protocol("WM_DELETE_WINDOW", self.cerrar)

    # ----- ajustes
    def carpeta_inicial(self):
        for d in (os.path.join(os.path.dirname(AQUI), "perfiles"), AQUI, os.path.dirname(AQUI), os.path.join(AQUI, "perfiles")):
            if glob.glob(os.path.join(d, "mw3_*.cfg")): return d
        d = os.path.join(AQUI, "perfiles_guardados"); os.makedirs(d, exist_ok=True); return d

    def guardar_ajustes(self):
        try: json.dump(self.aj, open(AJUSTES, "w"), indent=1)
        except OSError: pass

    def autodetectar(self):
        if not self.aj["storage"]:
            r = buscar_storage()
            if r: self.aj["storage"] = r[0]
        if os.name != "nt" and not self.aj["wine"]: self.aj["wine"] = buscar_wine()

    # ----- interfaz
    def _ui(self):
        izq = ttk.Frame(self, padding=6); izq.pack(side="left", fill="y")
        ttk.Label(izq, text="Perfiles (carpeta de perfiles)").pack(anchor="w")
        self.lbl_carpeta = ttk.Label(izq, text="", foreground="gray", wraplength=230); self.lbl_carpeta.pack(anchor="w")
        self.lista = tk.Listbox(izq, width=34, height=22, exportselection=False); self.lista.pack(fill="y", expand=True)
        self.lista.bind("<<ListboxSelect>>", self.al_elegir)
        fb = ttk.Frame(izq); fb.pack(fill="x", pady=3)
        for t, c in (("Nuevo", self.nuevo), ("Importar...", self.importar), ("Abrir...", self.abrir_otro)):
            ttk.Button(fb, text=t, command=c).pack(side="left", padx=1)
        fb = ttk.Frame(izq); fb.pack(fill="x")
        for t, c in (("Duplicar", self.duplicar), ("Eliminar", self.eliminar), ("Carpeta...", self.elegir_carpeta)):
            ttk.Button(fb, text=t, command=c).pack(side="left", padx=1)

        srv = ttk.LabelFrame(izq, text="Servidor", padding=6); srv.pack(fill="x", pady=8)
        self.lbl_estado = ttk.Label(srv, text="Detenido", foreground="gray"); self.lbl_estado.pack(anchor="w")
        f = ttk.Frame(srv); f.pack(fill="x", pady=2); ttk.Label(f, text="Puerto").pack(side="left")
        self.e_puerto = ttk.Entry(f, width=7); self.e_puerto.insert(0, self.aj["puerto"]); self.e_puerto.pack(side="left", padx=4)
        ttk.Button(srv, text="Iniciar con el perfil seleccionado", command=self.iniciar).pack(fill="x", pady=2)
        ttk.Button(srv, text="Detener servidor", command=self.detener).pack(fill="x")
        ttk.Button(srv, text="Ajustes de rutas...", command=self.ventana_ajustes).pack(fill="x", pady=(6, 0))

        der = ttk.Frame(self, padding=6); der.pack(side="right", fill="both", expand=True)
        arriba = ttk.Frame(der); arriba.pack(fill="x")
        ttk.Label(arriba, text="Archivo:").pack(side="left")
        ttk.Entry(arriba, textvariable=self.v["nombre"], width=28).pack(side="left", padx=4)
        ttk.Label(arriba, text=".cfg (se guarda como mw3_<nombre>.cfg)").pack(side="left")
        ttk.Button(arriba, text="Guardar", command=self.guardar).pack(side="right")
        self.lbl_cambios = ttk.Label(der, text="", foreground="#b36b00"); self.lbl_cambios.pack(anchor="w")
        nb = ttk.Notebook(der); nb.pack(fill="both", expand=True, pady=4)
        self.nb = nb
        self.t_general = ttk.Frame(nb, padding=8); nb.add(self.t_general, text="General")
        self.t_rot = ttk.Frame(nb, padding=8); nb.add(self.t_rot, text="Rotacion")
        self.t_bots = ttk.Frame(nb, padding=8); nb.add(self.t_bots, text="Bots")
        self.t_av = ttk.Frame(nb, padding=8); nb.add(self.t_av, text="Texto del archivo")
        self._general(); self._rotacion(); self._bots(); self._avanzado()
        lf = ttk.LabelFrame(der, text="Consola de Plutonium (console.log)", padding=4); lf.pack(fill="both", pady=2)
        self.log = tk.Text(lf, height=9, wrap="none", font=("monospace", 8), state="disabled"); self.log.pack(fill="both", expand=True)

    def fila(self, p, txt, var, ancho=24, opciones=None, **kw):
        f = ttk.Frame(p); f.pack(fill="x", pady=1)
        ttk.Label(f, text=txt, width=34).pack(side="left")
        if opciones: w = ttk.Combobox(f, values=opciones, textvariable=self.v[var], width=ancho - 2, state="readonly")
        else: w = ttk.Entry(f, textvariable=self.v[var], width=ancho, **kw)
        w.pack(side="left"); return w

    def _general(self):
        p = self.t_general
        g = ttk.LabelFrame(p, text="Servidor", padding=6); g.pack(fill="x")
        self.fila(g, "Nombre en el listado", "hostname")
        self.fila(g, "Maximo de jugadores (1-18)", "maxclients", 6)
        self.fila(g, "Contrasena (vacia = ninguna)", "password")
        self.fila(g, "Contrasena rcon", "rcon")
        self.fila(g, "Espacios privados", "privados", 6)
        self.fila(g, "Contrasena de espacios privados", "privpass")
        self.fila(g, "Chat de voz (0 no, 1 libre, 2 equipo)", "voz", 6, opciones=["0", "1", "2"])
        self.fila(g, "Votaciones (0 no, 1 si)", "votos", 6, opciones=["0", "1"])
        self.fila(g, "Muertos hablan con vivos (0/1)", "chat_muertos", 6, opciones=["0", "1"])
        self.fila(g, "Segundos para echar por inactividad", "inactividad", 8)
        self.fila(g, "Segundos de baneo temporal", "ban", 8)
        self.fila(g, "Proteccion contra flood (0/1)", "flood", 6, opciones=["0", "1"])
        g = ttk.LabelFrame(p, text="Partida (vacio = el valor del juego; se aplica a todos los modos de la rotacion)", padding=6)
        g.pack(fill="x", pady=6)
        self.fila(g, "Minutos por partida (0 = sin limite)", "tiempo", 8)
        self.fila(g, "Limite de puntos", "puntos", 8)
        self.fila(g, "Rondas (SD, SAB, DD)", "rondas", 8)
        self.fila(g, "Rondas para ganar (SD, SAB, DD)", "ganar", 8)
        self.fila(g, "Vidas por ronda (0 = ilimitadas)", "vidas", 8)
        g = ttk.LabelFrame(p, text="Reglas (nombres tomados del juego, efecto sin comprobar)", padding=6); g.pack(fill="x")
        self.fila(g, "Fuego amigo scr_team_fftype (0 no, 1 si, 2 reflejado, 3 compartido)", "fuego_amigo", 6, opciones=["", "0", "1", "2", "3"])
        self.fila(g, "Rachas scr_game_hardpoints (0 no, 1 si)", "rachas", 6, opciones=["", "0", "1"])
        self.fila(g, "Reaparicion forzada scr_player_forcerespawn", "respawn_forzado", 6, opciones=["", "0", "1"])

    def _rotacion(self):
        p = self.t_rot
        f = ttk.Frame(p); f.pack(fill="x")
        self.c_modo = ttk.Combobox(f, values=[etiqueta(MODOS, k) for k in MODOS], state="readonly", width=26)
        self.c_modo.set(etiqueta(MODOS, "war")); self.c_modo.pack(side="left")
        self.c_mapa = ttk.Combobox(f, values=[etiqueta(MAPAS, k) for k in MAPAS], state="readonly", width=30)
        self.c_mapa.set(etiqueta(MAPAS, "mp_seatown")); self.c_mapa.pack(side="left", padx=4)
        ttk.Button(f, text="Agregar", command=self.agregar).pack(side="left")
        f = ttk.Frame(p); f.pack(fill="x", pady=4)
        for t, c in (("Todos los mapas base con este modo", lambda: self.agregar_varios(BASE)),
                     ("Solo mapas Plutonium con este modo", lambda: self.agregar_varios(EXTRA)),
                     ("Subir", lambda: self.mover(-1)), ("Bajar", lambda: self.mover(1)),
                     ("Quitar", self.quitar), ("Vaciar", self.vaciar)):
            ttk.Button(f, text=t, command=c).pack(side="left", padx=2)
        self.l_rot = tk.Listbox(p, height=18, exportselection=False); self.l_rot.pack(fill="both", expand=True)
        ttk.Label(p, text="Al llegar al final vuelve a empezar. Los mapas Plutonium los descarga Plutonium aparte; los demas jugadores los necesitan.",
                  foreground="gray", wraplength=800).pack(anchor="w", pady=2)

    def _bots(self):
        p = self.t_bots
        g = ttk.LabelFrame(p, text="Cantidad", padding=6); g.pack(fill="x")
        for val, txt in (("ninguno", "Sin bots"), ("fill", "Mantener N jugadores en total (humanos + bots)"),
                         ("add", "Agregar N bots fijos")):
            ttk.Radiobutton(g, text=txt, value=val, variable=self.v["bots"]).pack(anchor="w")
        self.fila(g, "N", "bots_n", 6)
        self.fila(g, "Modo de relleno", "fillmode", 26, opciones=[m[0] for m in FILL_MODOS])
        ttk.Checkbutton(g, text="Sacar bots cuando entran humanos", variable=self.v["kick"], onvalue="1", offvalue="0").pack(anchor="w")
        ttk.Checkbutton(g, text="Contar a quien este en espectador", variable=self.v["spec"], onvalue="1", offvalue="0").pack(anchor="w")
        ttk.Checkbutton(g, text="Solo hay bots mientras haya humanos", variable=self.v["watch"], onvalue="1", offvalue="0").pack(anchor="w")
        g = ttk.LabelFrame(p, text="Dificultad y equipos", padding=6); g.pack(fill="x", pady=6)
        self.fila(g, "Dificultad (0 aleatoria, 1 facil ... 7 dificil, 9 totalmente aleatoria)", "skill", 6, opciones=[s.split()[0] for s in SKILLS])
        self.fila(g, "Dificultad minima (con 0)", "skmin", 6)
        self.fila(g, "Dificultad maxima (con 0)", "skmax", 6)
        self.fila(g, "Equipo de los bots", "equipo", 14, opciones=EQUIPOS)
        self.fila(g, "Bots forzados al equipo axis", "team_n", 6)
        self.fila(g, "Al repartir equipos cuenta solo bots (0/1)", "team_mode", 6, opciones=["0", "1"])
        g = ttk.LabelFrame(p, text="Comportamiento (desmarcar = el bot no lo hace)", padding=6); g.pack(fill="x")
        for i, n in enumerate(PLAY):
            ttk.Checkbutton(g, text=n, variable=self.play[n], onvalue="1", offvalue="0").grid(row=i // 4, column=i % 4, sticky="w", padx=6)

    def _avanzado(self):
        p = self.t_av
        ttk.Label(p, text="Lineas extra (cualquier otra variable que quieras; se agregan al final):").pack(anchor="w")
        self.extras = tk.Text(p, height=5, font=("monospace", 9)); self.extras.pack(fill="x")
        f = ttk.Frame(p); f.pack(fill="x", pady=4)
        ttk.Label(p, text="Texto completo del archivo (editable a mano):").pack(anchor="w")
        self.texto = tk.Text(p, wrap="none", font=("monospace", 9)); self.texto.pack(fill="both", expand=True)
        self.texto.bind("<Key>", lambda e: self.marcar_manual() if e.keysym not in ("Left", "Right", "Up", "Down", "Prior", "Next", "Home", "End") and not (e.state & 4) else None)
        f = ttk.Frame(p); f.pack(fill="x", pady=3)
        ttk.Button(f, text="Regenerar texto desde el formulario", command=self.regenerar).pack(side="left")
        ttk.Button(f, text="Leer el texto al formulario", command=self.leer_texto).pack(side="left", padx=6)

    # ----- estado del formulario / texto
    def valores(self):
        c = {k: v.get().strip() for k, v in self.v.items()}
        c["play"] = {p: v.get() for p, v in self.play.items()}
        c["rotacion"] = self.rotacion
        c["extras"] = self.extras.get("1.0", "end").strip("\n")
        for k in TEXTO: c[k] = c[k].replace('"', "")
        c["fillmode"] = c["fillmode"][:1]
        return c

    def cambio(self):
        if self.cargando: return
        self.sucio(True)
        if not self.manual: self.regenerar(sin_marcar=True)
        else: self.lbl_cambios.config(text="El texto esta editado a mano: el formulario ya no lo actualiza. Usa 'Regenerar' o 'Leer el texto'.")

    def _mod_extras(self, e=None):
        if self.extras.edit_modified():
            self.extras.edit_modified(False)
            if not self.cargando: self.cambio()

    def marcar_manual(self):
        self.manual = True; self.sucio(True)
        self.lbl_cambios.config(text="Texto editado a mano: se guarda tal cual. Usa 'Leer el texto' para pasarlo al formulario.")

    def sucio(self, s): self._sucio = s; self.title("Gestor de servidor MW3 LAN" + (" *" if s else ""))

    def poner_texto(self, t):
        self.texto.delete("1.0", "end"); self.texto.insert("1.0", t)

    def regenerar(self, sin_marcar=False):
        self.manual = False; self.lbl_cambios.config(text="")
        self.poner_texto(construir(self.valores()))
        if not sin_marcar: self.sucio(True)

    def cargar_texto(self, t):
        v, rot, play, extras = parsear(t)
        self.cargando = True
        for k, val in v.items():
            if k in self.v: self.v[k].set(val)
        for p, val in play.items(): self.play[p].set(val)
        self.rotacion = rot
        self.extras.delete("1.0", "end"); self.extras.insert("1.0", extras); self.extras.edit_modified(False)
        self.cargando = False
        self.pintar_rot()

    def leer_texto(self):
        self.cargar_texto(self.texto.get("1.0", "end")); self.manual = False; self.lbl_cambios.config(text="")

    # ----- rotacion
    def pintar_rot(self):
        self.l_rot.delete(0, "end")
        for i, (m, mp) in enumerate(self.rotacion, 1): self.l_rot.insert("end", f"{i:2}. {nombre(MODOS, m)}  -  {nombre(MAPAS, mp)}")

    def tras_rot(self):
        self.pintar_rot(); self.cambio()

    def agregar(self):
        self.rotacion.append((codigo(self.c_modo.get()), codigo(self.c_mapa.get()))); self.tras_rot()

    def agregar_varios(self, d):
        self.rotacion += [(codigo(self.c_modo.get()), m) for m in d]; self.tras_rot()

    def quitar(self):
        for i in reversed(self.l_rot.curselection()): del self.rotacion[i]
        self.tras_rot()

    def vaciar(self): self.rotacion.clear(); self.tras_rot()

    def mover(self, d):
        s = self.l_rot.curselection()
        if not s: return
        i, j = s[0], s[0] + d
        if 0 <= j < len(self.rotacion):
            self.rotacion[i], self.rotacion[j] = self.rotacion[j], self.rotacion[i]; self.tras_rot(); self.l_rot.selection_set(j)

    # ----- perfiles (archivos)
    def ruta(self, nom): return os.path.join(self.aj["perfiles"], f"mw3_{nom}.cfg")

    def listar(self):
        self.lbl_carpeta.config(text=self.aj["perfiles"])
        self.archivos = sorted(f for f in os.listdir(self.aj["perfiles"]) if f.lower().endswith(".cfg") and f.lower() != "server.cfg")
        self.lista.delete(0, "end")
        for f in self.archivos: self.lista.insert("end", ("▶ " if f == self.perfil_activo else "  ") + f)
        if self.archivo and os.path.basename(self.archivo) in self.archivos:
            i = self.archivos.index(os.path.basename(self.archivo)); self.lista.selection_set(i)

    def confirmar_cambios(self):
        if getattr(self, "_sucio", False):
            r = messagebox.askyesnocancel("Cambios sin guardar", "Hay cambios sin guardar en el perfil actual. ¿Guardarlos?")
            if r is None: return False
            if r: return self.guardar()
        return True

    def al_elegir(self, e=None):
        s = self.lista.curselection()
        if not s: return
        ruta = os.path.join(self.aj["perfiles"], self.archivos[s[0]])
        if ruta == self.archivo: return
        if not self.confirmar_cambios(): return self.listar()
        self.abrir(ruta)

    def abrir(self, ruta):
        try: t = open(ruta, encoding="utf8", errors="replace").read()
        except OSError as e: return messagebox.showerror("No se pudo abrir", str(e))
        self.cargar_texto(t); self.poner_texto(t); self.manual = False; self.lbl_cambios.config(text="")
        self.archivo = ruta
        n = os.path.basename(ruta)[:-4]
        self.v["nombre"].set(n[4:] if n.startswith("mw3_") else n)
        self.sucio(False); self.aj["ultimo"] = os.path.basename(ruta); self.listar()

    def abrir_otro(self):
        if not self.confirmar_cambios(): return
        r = filedialog.askopenfilename(title="Abrir un .cfg", filetypes=[("Configuracion", "*.cfg"), ("Todos", "*")])
        if r: self.abrir(r)

    def importar(self):
        rs = filedialog.askopenfilenames(title="Importar .cfg a la carpeta de perfiles", filetypes=[("Configuracion", "*.cfg")])
        for r in rs:
            d = os.path.join(self.aj["perfiles"], os.path.basename(r))
            if os.path.exists(d) and not messagebox.askyesno("Ya existe", f"{os.path.basename(d)} ya existe. ¿Sobrescribir?"): continue
            shutil.copy(r, d)
        self.listar()

    def nuevo(self):
        if not self.confirmar_cambios(): return
        self.archivo = None; self.cargar_texto(""); self.v["nombre"].set("mi_perfil")
        self.rotacion = [("war", "mp_seatown"), ("war", "mp_dome")]; self.pintar_rot()
        self.regenerar(); self.sucio(True)

    def duplicar(self):
        if not self.archivo: return messagebox.showinfo("Duplicar", "Elige primero un perfil.")
        n = simpledialog.askstring("Duplicar", "Nombre del nuevo perfil (sin mw3_ ni .cfg):", parent=self)
        if not n: return
        if not re.fullmatch(r"[A-Za-z0-9_-]+", n): return messagebox.showerror("Nombre", "Solo letras, numeros, _ y -.")
        d = self.ruta(n)
        if os.path.exists(d): return messagebox.showerror("Ya existe", os.path.basename(d))
        shutil.copy(self.archivo, d); self.abrir(d)

    def eliminar(self):
        s = self.lista.curselection()
        if not s: return
        f = os.path.join(self.aj["perfiles"], self.archivos[s[0]])
        if messagebox.askyesno("Eliminar", f"¿Eliminar {os.path.basename(f)}? No se puede deshacer."):
            os.remove(f)
            if f == self.archivo: self.archivo = None
            self.listar()

    def elegir_carpeta(self):
        d = filedialog.askdirectory(title="Carpeta de perfiles", initialdir=self.aj["perfiles"])
        if d and self.confirmar_cambios(): self.aj["perfiles"] = d; self.archivo = None; self.guardar_ajustes(); self.listar()

    def validar(self, c):
        if not re.fullmatch(r"[A-Za-z0-9_-]+", c["nombre"]): return "El nombre del archivo solo admite letras, numeros, _ y -."
        if not self.manual:
            if not c["rotacion"]: return "Agrega al menos un mapa a la rotacion."
            if not c["maxclients"].isdigit() or not 1 <= int(c["maxclients"]) <= 18: return "Maximo de jugadores: de 1 a 18."
            for k in LIMITES:
                if c[k] != "" and not re.fullmatch(r"\d+(\.\d+)?", c[k]): return f"'{k}' debe ser un numero o estar vacio."
            if c["bots"] != "ninguno" and not c["bots_n"].isdigit(): return "N de bots debe ser un numero."
        return None

    def guardar(self):
        c = self.valores(); err = self.validar(c)
        if err: messagebox.showerror("Revisa los datos", err); return False
        destino = self.ruta(c["nombre"])
        if os.path.abspath(destino) != os.path.abspath(self.archivo or "") and os.path.exists(destino) \
                and not messagebox.askyesno("Ya existe", f"{os.path.basename(destino)} ya existe. ¿Sobrescribir?"): return False
        try:
            with open(destino, "w", newline="\n", encoding="utf8") as f: f.write(self.texto.get("1.0", "end").rstrip("\n") + "\n")
        except OSError as e: messagebox.showerror("No se pudo guardar", str(e)); return False
        self.archivo = destino; self.aj["ultimo"] = os.path.basename(destino); self.sucio(False); self.listar()
        return True

    # ----- servidor
    def ventana_ajustes(self):
        w = tk.Toplevel(self); w.title("Ajustes de rutas"); w.geometry("760x300"); w.transient(self)
        f = ttk.Frame(w, padding=10); f.pack(fill="both", expand=True)
        self.w_aj = {}
        filas = [("storage", "Carpeta storage/iw5 de Plutonium", True), ("mw3", "Carpeta de MW3 (vacio = la de Plutonium)", True)]
        if os.name != "nt": filas.append(("wine", "Programa wine", False))
        for k, t, es_dir in filas:
            ttk.Label(f, text=t).pack(anchor="w")
            r = ttk.Frame(f); r.pack(fill="x", pady=(0, 6))
            e = ttk.Entry(r, width=80); e.insert(0, self.aj[k]); e.pack(side="left"); self.w_aj[k] = e
            def elegir(e=e, d=es_dir):
                x = filedialog.askdirectory() if d else filedialog.askopenfilename()
                if x: e.delete(0, "end"); e.insert(0, x)
            ttk.Button(r, text="...", width=3, command=elegir).pack(side="left", padx=3)
            if k == "storage":
                self.cb_storage = ttk.Combobox(f, width=100); self.cb_storage.pack(anchor="w", pady=(0, 6))
                self.cb_storage.bind("<<ComboboxSelected>>", lambda e: (self.w_aj["storage"].delete(0, "end"), self.w_aj["storage"].insert(0, self.cb_storage.get())))
                ttk.Button(f, text="Buscar instalaciones de Plutonium", command=lambda: self.cb_storage.config(values=buscar_storage())).pack(anchor="w", pady=(0, 6))
        def ok():
            for k, e in self.w_aj.items(): self.aj[k] = e.get().strip()
            self.guardar_ajustes(); w.destroy()
        ttk.Button(f, text="Guardar", command=ok).pack(anchor="e")

    def datos_servidor(self):
        st = self.aj["storage"]
        if not st or not os.path.isdir(st): raise ValueError("Falta la carpeta storage/iw5 de Plutonium (Ajustes de rutas).")
        plu = os.path.dirname(os.path.dirname(st))
        exe = os.path.join(plu, "bin", "plutonium-bootstrapper-win32.exe")
        if not os.path.exists(exe): raise ValueError(f"No existe {exe}")
        mw3 = self.aj["mw3"]
        if not mw3:
            try: mw3 = json.load(open(os.path.join(plu, "config.json")))["iw5Path"]
            except Exception: raise ValueError("No pude leer iw5Path de config.json de Plutonium. Indica la carpeta de MW3 en Ajustes.")
        if os.name != "nt" and not re.match(r"^[A-Za-z]:[\\/]", mw3): mw3 = "Z:" + mw3.replace("/", "\\")
        return st, plu, exe, mw3

    def iniciar(self):
        if self.proc and self.proc.poll() is None: return messagebox.showinfo("Servidor", "Ya hay un servidor iniciado desde esta ventana. Detenlo primero.")
        if not self.archivo: return messagebox.showinfo("Servidor", "Elige o guarda un perfil primero.")
        if getattr(self, "_sucio", False) and not self.confirmar_cambios(): return
        puerto = self.e_puerto.get().strip()
        if not puerto.isdigit(): return messagebox.showerror("Puerto", "El puerto debe ser un numero.")
        try: st, plu, exe, mw3 = self.datos_servidor()
        except ValueError as e: return messagebox.showerror("Servidor", str(e))
        if not puerto_libre(puerto):
            return messagebox.showerror("Puerto en uso", f"El puerto UDP {puerto} ya esta en uso: hay otro servidor abierto. Cierralo o usa otro puerto.")
        sc = os.path.join(st, "server.cfg")
        try:
            if os.path.exists(sc): shutil.copy(sc, sc + time.strftime(".bak-%Y%m%d-%H%M%S"))
            shutil.copy(self.archivo, sc)
        except OSError as e: return messagebox.showerror("server.cfg", str(e))
        args = ["iw5mp", mw3, "-lan", "-dedicated", "+set", "net_port", puerto, "+map_rotate"]
        env = dict(os.environ)
        if os.name == "nt":
            cmd, kw = [exe, *args], {}
        else:
            wine = self.aj["wine"] or buscar_wine()
            m = re.match(r"^(.*)/drive_c/users/([^/]+)/", st)
            if not wine or not m: return messagebox.showerror("Servidor", "Falta wine o no pude deducir el prefijo desde la ruta de storage. Revisa Ajustes de rutas.")
            env["WINEPREFIX"] = m.group(1); env["WINEDEBUG"] = "-all"
            cmd = [wine, f"C:\\users\\{m.group(2)}\\AppData\\Local\\Plutonium\\bin\\plutonium-bootstrapper-win32.exe", *args]
            kw = {"start_new_session": True}
        try:
            self.proc = subprocess.Popen(cmd, cwd=plu, env=env, stdout=open(os.path.join(AQUI, "gestor_servidor.log"), "w"), stderr=subprocess.STDOUT, **kw)
        except OSError as e: return messagebox.showerror("No se pudo iniciar", str(e))
        self.perfil_activo = os.path.basename(self.archivo); self.aj["puerto"] = puerto; self.guardar_ajustes(); self.listar()

    def detener(self):
        if not (self.proc and self.proc.poll() is None):
            return messagebox.showinfo("Servidor", "No hay un servidor iniciado desde esta ventana. Si lo abriste de otra forma, cierralo desde su ventana.")
        try:
            if os.name == "nt": self.proc.terminate()
            else: os.killpg(self.proc.pid, signal.SIGTERM)
        except OSError as e: messagebox.showerror("Servidor", str(e))

    def tic(self):
        corriendo = self.proc is not None and self.proc.poll() is None
        if corriendo: self.lbl_estado.config(text=f"En marcha: {self.perfil_activo} (puerto {self.aj['puerto']})", foreground="#1a7f37")
        else:
            if self.perfil_activo: self.perfil_activo = None; self.listar()
            self.lbl_estado.config(text="Detenido", foreground="gray")
        try:
            plu = os.path.dirname(os.path.dirname(self.aj["storage"]))
            with open(os.path.join(plu, "console.log"), "rb") as f:
                f.seek(0, 2); n = f.tell(); f.seek(max(0, n - 12000)); t = sin_ansi(f.read().decode("utf8", "replace"))
            t = "\n".join(t.splitlines()[-60:])
            if t != getattr(self, "_ult", None):
                self._ult = t; self.log.config(state="normal"); self.log.delete("1.0", "end"); self.log.insert("end", t)
                self.log.see("end"); self.log.config(state="disabled")
        except Exception: pass
        self.after(2000, self.tic)

    def cerrar(self):
        if not self.confirmar_cambios(): return
        if self.proc and self.proc.poll() is None and not messagebox.askyesno("Servidor en marcha", "El servidor sigue abierto y seguira corriendo al cerrar esta ventana. ¿Cerrar igual?"): return
        self.aj["puerto"] = self.e_puerto.get().strip() or "5000"; self.guardar_ajustes(); self.destroy()


if __name__ == "__main__":
    App().mainloop()

#!/usr/bin/env python3
"""Parcha z_svr_bots.iwd (Bot Warfare 2.3.0) para juegos base mas viejos que Plutonium r5369.
Uso: python3 parchar_iwd.py entrada.iwd salida.iwd [--grnd] [--buff]   (sin flags aplica los dos)
  --grnd: cambia grnd::isingrindzone() por self.ingrindzone (esa funcion no existe en el juego base).
  --buff: cambia isvalidweaponbuff(...) por true (el juego base la define con otros argumentos)."""
import sys, zipfile

args = [a for a in sys.argv[1:] if not a.startswith("--")]
flags = {a for a in sys.argv[1:] if a.startswith("--")} or {"--grnd", "--buff"}
if len(args) != 2: sys.exit(__doc__)
cambios = 0
with zipfile.ZipFile(args[0]) as zi, zipfile.ZipFile(args[1], "w", zipfile.ZIP_STORED) as zo:
    for i in zi.infolist():
        d = zi.read(i.filename)
        if i.filename in ("maps/mp/bots/_bot_script.gsc", "maps/mp/bots/_bot_utility.gsc"):
            s = d.decode("utf8", "surrogateescape")
            if "--grnd" in flags:
                for q in ("self", "player"):
                    o = q + r" maps\mp\gametypes\grnd::isingrindzone()"
                    cambios += s.count(o)
                    s = s.replace(o, f"( isdefined( {q}.ingrindzone ) && {q}.ingrindzone )")
            if "--buff" in flags:
                for a in ("loadoutPrimaryBuff, loadoutPrimary", "loadoutSecondaryBuff, loadoutSecondary"):
                    o = rf"maps\mp\gametypes\_class::isvalidweaponbuff( {a} )"
                    cambios += s.count(o)
                    s = s.replace(o, "true")
            d = s.encode("utf8", "surrogateescape")
        zo.writestr(i, d)
print(f"{cambios} reemplazos -> {args[1]}")

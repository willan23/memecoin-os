"""Write named .env values to temp files. Prints only names and lengths."""
from pathlib import Path
import os
import sys

want = [n.upper() for n in sys.argv[1:]]
outdir = Path(os.environ.get("TEMP", "."))
text = Path(".env").read_text(encoding="utf-8-sig")
got = {}
for line in text.splitlines():
    if "=" not in line:
        continue
    name, val = line.split("=", 1)
    key = name.strip().upper()
    if key in want:
        got[key] = val.strip().strip('"').strip("'")
for key in want:
    val = got.get(key, "")
    if not val:
        print(f"{key} missing")
        sys.exit(1)
    dest = outdir / f"mcos-{key.lower().replace('_', '-')}.txt"
    dest.write_text(val, encoding="ascii")
    print(f"{key} extracted len={len(val)}")

"""Print whether named env keys are set. Never prints values."""
from pathlib import Path
import sys

want = {n.upper() for n in sys.argv[1:]}
text = Path(".env").read_text(encoding="utf-8-sig")
found = {n: False for n in want}
for line in text.splitlines():
    if "=" not in line:
        continue
    name, val = line.split("=", 1)
    key = name.strip().upper()
    if key in want:
        found[key] = bool(val.strip().strip('"').strip("'"))
        print(f"{key} set={found[key]} len={len(val.strip().strip(chr(34)).strip(chr(39)))}")
for n in want:
    if n not in {line.split("=", 1)[0].strip().upper() for line in text.splitlines() if "=" in line}:
        print(f"{n} set=False missing")

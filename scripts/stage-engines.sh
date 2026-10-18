#!/usr/bin/env bash
# Download pinned engines for one triple and copy the headless binaries into engines/bin/.
# Usage: scripts/stage-engines.sh windows-x86_64|darwin-aarch64|darwin-x86_64|linux-x86_64|android-aarch64
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TRIPLE="${1:-}"
if [[ -z "$TRIPLE" ]]; then
  echo "usage: $0 <triple>" >&2
  exit 2
fi
LOCK="$ROOT/policy/engines.lock"
DEST="$ROOT/engines/bin"
CACHE="$ROOT/engines/cache"
mkdir -p "$DEST" "$CACHE"

if command -v python3 >/dev/null 2>&1; then
  PY=python3
elif command -v python >/dev/null 2>&1; then
  PY=python
else
  echo "python3 is required" >&2
  exit 1
fi
"$PY" - "$LOCK" "$DEST" "$CACHE" "$TRIPLE" <<'PY'
import json, os, shutil, sys, tarfile, zipfile, urllib.request

lock_path, dest, cache, triple = sys.argv[1:5]
lock = json.load(open(lock_path))
windows = triple.startswith("windows-")

# Canonical names WNY looks up in engines/bin (plus Windows companions).
WANTED = {
    "dnscrypt-proxy": ["dnscrypt-proxy.exe" if windows else "dnscrypt-proxy"],
    "goodbyedpi": ["goodbyedpi.exe", "WinDivert.dll", "WinDivert64.sys", "WinDivert32.sys"],
    "spoofdpi": ["spoofdpi.exe" if windows else "spoofdpi"],
    "byedpi": ["ciadpi.exe" if windows else "ciadpi"],
    "tor": (
        ["tor.exe", "lyrebird.exe"] if windows else ["tor", "lyrebird"]
    ),
    "hev-socks5-tunnel": ["hev-socks5-tunnel.exe" if windows else "hev-socks5-tunnel"],
}

SKIP_URL_SUBSTR = ("master.zip", "/archive/refs/heads/")


def fetch(url, path):
    if os.path.isfile(path) and os.path.getsize(path) > 0:
        print(f"cached {os.path.basename(path)}")
        return
    print(f"fetch {url}")
    req = urllib.request.Request(url, headers={"User-Agent": "WNY-stage-engines"})
    with urllib.request.urlopen(req, timeout=180) as r, open(path, "wb") as f:
        shutil.copyfileobj(r, f)


def prefer_path(path: str) -> int:
    p = path.replace("\\", "/").lower()
    score = 0
    if "/debug/" in p or p.startswith("debug/"):
        score -= 50
    if any(x in p for x in ("/x86_64/", "/x64/", "/amd64/", "/win64/")):
        score += 20
    if any(x in p for x in ("/x86/", "/i386/", "/win32/", "/x86_32/")):
        score -= 10
    return score


def is_lib(name: str) -> bool:
    n = name.lower()
    return (
        n.endswith((".dll", ".dylib", ".sys"))
        or n.endswith(".so")
        or ".so." in n
    )


def match_canon(basename: str, wanted: dict) -> str | None:
    b = basename.lower()
    if b in wanted:
        return wanted[b]
    skip = (".txt", ".md", ".toml", ".json", ".cmd", ".bat", ".pem")
    if any(b.endswith(s) for s in skip):
        return None
    for key, canon in wanted.items():
        stem = key.replace(".exe", "")
        if b == stem or b.startswith(stem + "-") or b.startswith(stem + "_"):
            return canon
    return None


def write_bin(out_path: str, data: bytes):
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    with open(out_path, "wb") as f:
        f.write(data)
    os.chmod(out_path, 0o755)


def extract_wanted(archive_path: str, names: list[str]) -> list[str]:
    wanted = {n.lower(): n for n in names}
    found = []

    def take_from_archive(members):
        # members: list of (member_name, reader)
        best = {}
        for member_name, reader in members:
            base = os.path.basename(member_name.rstrip("/\\"))
            canon = match_canon(base, wanted)
            if not canon:
                continue
            score = prefer_path(member_name)
            prev = best.get(canon)
            if prev is None or score > prev[0]:
                best[canon] = (score, member_name, reader)
        for canon, (_score, member_name, reader) in best.items():
            write_bin(os.path.join(dest, canon), reader(member_name))
            found.append(canon)
            print(f"  staged {canon} <- {member_name}")
        # Shared libs next to tor / goodbyedpi so $ORIGIN / DLL search works.
        for canon, (_score, member_name, reader) in best.items():
            parent = os.path.dirname(member_name.replace("\\", "/"))
            for sibling, sreader in members:
                s = sibling.replace("\\", "/")
                if os.path.dirname(s) != parent:
                    continue
                base = os.path.basename(s)
                if not is_lib(base):
                    continue
                out = os.path.join(dest, base)
                if os.path.isfile(out):
                    continue
                write_bin(out, sreader(sibling))
                found.append(base)
                print(f"  staged {base} <- {sibling}")

    if zipfile.is_zipfile(archive_path):
        z = zipfile.ZipFile(archive_path)

        def zread(n, z=z):
            return z.read(n)

        take_from_archive(
            [(info.filename, zread) for info in z.infolist() if not info.is_dir()]
        )
        return found

    try:
        t = tarfile.open(archive_path, "r:*")
    except tarfile.TarError:
        t = None
    if t is not None:
        def tread(n, t=t):
            f = t.extractfile(n)
            if f is None:
                return b""
            return f.read()

        take_from_archive(
            [(info.name, tread) for info in t.getmembers() if info.isfile()]
        )
        return found

    # Raw binary (hev darwin/linux/android artifacts have no archive wrapper).
    canon = names[0]
    shutil.copy2(archive_path, os.path.join(dest, canon))
    os.chmod(os.path.join(dest, canon), 0o755)
    print(f"  staged {canon} <- raw")
    return [canon]


staged_any = False
errors = []
for eng in lock["engines"]:
    eid = eng["id"]
    arts = eng.get("artifacts") or {}
    meta = arts.get(triple)
    if not meta:
        continue
    url = (meta.get("url") or "").strip()
    if not url:
        continue
    if any(s in url for s in SKIP_URL_SUBSTR):
        print(f"skip {eid} {triple}: unpinned url")
        continue
    names = WANTED.get(eid)
    if not names:
        print(f"skip {eid}: no binary map")
        continue
    name = os.path.basename(url.split("?")[0]) or f"{eid}.bin"
    path = os.path.join(cache, f"{eid}-{triple}-{name}")
    try:
        fetch(url, path)
    except Exception as e:
        errors.append(f"{eid}: download failed: {e}")
        print(f"  failed {eid}: {e}")
        continue
    got = extract_wanted(path, names)
    if not got:
        errors.append(f"{eid}: archive had none of {names}")
        print(f"  no binaries in {name}")
        continue
    staged_any = True

if not staged_any:
    print("error: no engines staged", file=sys.stderr)
    sys.exit(1)
if errors:
    print("warnings:")
    for e in errors:
        print(" ", e)
print("staged into", dest)
for fn in sorted(os.listdir(dest)):
    p = os.path.join(dest, fn)
    if os.path.isfile(p):
        print(f"  {fn} {os.path.getsize(p)}")
PY

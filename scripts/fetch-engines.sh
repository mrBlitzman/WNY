#!/usr/bin/env bash
# Maintainer: download pinned upstream engines, hash them, update policy/engines.lock.
# Runtime WNY does not call this. GitHub may be blocked for users.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOCK="$ROOT/policy/engines.lock"
DEST="$ROOT/engines/cache"
mkdir -p "$DEST"

python3 - <<'PY' "$LOCK" "$DEST"
import hashlib, json, os, sys, urllib.request
lock_path, dest = sys.argv[1], sys.argv[2]
lock = json.load(open(lock_path))
changed = False
for eng in lock["engines"]:
    arts = eng.get("artifacts") or {}
    for triple, meta in arts.items():
        url = meta.get("url") or ""
        if not url or "latest" in url or url.endswith("master.zip"):
            print(f"skip {eng['id']} {triple}: unpinned url")
            continue
        name = os.path.basename(url.split("?")[0])
        path = os.path.join(dest, f"{eng['id']}-{triple}-{name}")
        if not os.path.isfile(path):
            print(f"fetch {url}")
            try:
                urllib.request.urlretrieve(url, path)
            except Exception as e:
                print(f"  failed: {e}")
                continue
        h = hashlib.sha256(open(path, "rb").read()).hexdigest()
        if meta.get("sha256") != h:
            meta["sha256"] = h
            changed = True
            print(f"  {eng['id']} {triple} {h}")
if changed:
    json.dump(lock, open(lock_path, "w"), indent=2)
    print("updated", lock_path)
else:
    print("hashes unchanged")
PY

echo "Copy the matching artifact into engines/bin/ next to the WNY binary before shipping."

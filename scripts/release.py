#!/usr/bin/env python3
"""Build dist/baddel.zip from extension/ and keep docs/shared.js in sync."""
import json
import os
import shutil
import zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXT = os.path.join(ROOT, "extension")
DIST = os.path.join(ROOT, "dist")
SKIP = {"README.txt"}  # the repo README replaces it


def main():
    with open(os.path.join(EXT, "manifest.json"), encoding="utf-8") as f:
        version = json.load(f)["version"]

    # the website demo must use exactly the same conversion map as the extension
    shutil.copyfile(os.path.join(EXT, "shared.js"), os.path.join(ROOT, "docs", "shared.js"))

    os.makedirs(DIST, exist_ok=True)
    out = os.path.join(DIST, "baddel.zip")
    if os.path.exists(out):
        os.remove(out)

    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        for root, _, files in os.walk(EXT):
            for name in sorted(files):
                if name in SKIP:
                    continue
                path = os.path.join(root, name)
                z.write(path, arcname="baddel/" + os.path.relpath(path, EXT).replace(os.sep, "/"))
        z.writestr("baddel/README.txt", README)

    # served by GitHub Pages and via raw.githubusercontent, so no GitHub Release is needed
    site_dl = os.path.join(ROOT, "docs", "downloads")
    os.makedirs(site_dl, exist_ok=True)
    shutil.copyfile(out, os.path.join(site_dl, "baddel.zip"))

    print(f"built {out} (v{version}) and copied to docs/downloads/")


README = """Baddel - Arabic <-> English keyboard fixer

Install (Chrome / Edge / Brave / Opera):
1. Extract this ZIP somewhere permanent.
2. Open chrome://extensions (edge://extensions) and turn on Developer mode.
3. Click "Load unpacked" and choose the "baddel" folder.

Use: select text typed in the wrong language and press Ctrl+Shift+X
(change the shortcut from the extension's settings page).

Website:  https://shawady6.github.io/baddel/
Source:   https://github.com/shawady6/baddel
License:  MIT
"""

if __name__ == "__main__":
    main()

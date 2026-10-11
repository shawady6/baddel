// Checks that desktop/windows/baddel.ahk uses exactly the conversion map of the extension.
// It parses the generated AutoHotkey tables back and replays the script's Convert()
// algorithm in JS, comparing it with extension/shared.js.
const fs = require("fs");
const path = require("path");

module.exports = function run(t) {
  const root = path.join(__dirname, "..");
  const A = require(path.join(root, "extension", "shared.js"));
  const gen = require(path.join(root, "scripts", "gen_ahk.js"));
  const raw = fs.readFileSync(gen.file, "utf8");
  const src = raw.replace(/^﻿/, "");

  t("ahk: saved as UTF-8 with BOM", raw.charCodeAt(0) === 0xfeff);
  t("ahk: tables are up to date (run node scripts/gen_ahk.js)",
    src.includes(";@@MAPS_BEGIN\n" + gen.build() + "\n;@@MAPS_END"));

  // ---- parse the AHK tables back, undoing the AHK string escapes
  const unesc = (s) => s.replace(/""/g, '"').replace(/``/g, "`");
  const enToAr = new Map(), arToEn = new Map();
  const re = /^(enToAr|arToEn)\["((?:[^"]|"")*)"\] := "((?:[^"]|"")*)"$/;
  for (const line of src.split("\n")) {
    const m = re.exec(line);
    if (!m) continue;
    (m[1] === "enToAr" ? enToAr : arToEn).set(unesc(m[2]), unesc(m[3]));
  }
  t("ahk: parsed both tables", enToAr.size > 50 && arToEn.size > 40);
  t("ahk: special characters survive escaping", enToAr.get("`") === "ذ" && enToAr.get("'") === "ط");

  // ---- replay of Convert() from baddel.ahk
  const ahkConvert = (text, mode) => {
    if (mode === "auto") {
      const ar = (text.match(/[؀-ۿ]/g) || []).length;
      const la = (text.match(/[A-Za-z]/g) || []).length;
      if (!ar && !la) return text;
      mode = ar >= la ? "ar2en" : "en2ar";
    }
    const map = mode === "ar2en" ? arToEn : enToAr;
    let out = "";
    for (const ch of text) out += map.has(ch) ? map.get(ch) : ch;
    return out;
  };

  // deterministic pseudo-random strings drawn from printable ASCII and the Arabic block
  let seed = 12345;
  const rnd = () => (seed = (seed * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff;
  const pool = [];
  for (let c = 32; c <= 126; c++) pool.push(String.fromCharCode(c));
  for (let c = 0x0621; c <= 0x064a; c++) pool.push(String.fromCharCode(c));
  pool.push("؟", "،", "؛", "ـ", "\n");
  let same = true, count = 0;
  for (let i = 0; i < 3000 && same; i++) {
    let s = "";
    const n = 1 + Math.floor(rnd() * 14);
    for (let k = 0; k < n; k++) s += pool[Math.floor(rnd() * pool.length)];
    for (const mode of ["auto", "ar2en", "en2ar"]) {
      if (ahkConvert(s, mode) !== A.convert(s, mode)) { same = false; console.log("  mismatch:", JSON.stringify(s), mode); break; }
      count++;
    }
  }
  t(`ahk: Convert() matches the extension on ${count} random cases`, same);

  const examples = [["hgsghl ugd;l", "السلام عليكم"], ["lvpfh", "مرحبا"], ["اثممخ", "hello"]];
  t("ahk: known examples", examples.every(([i, o]) => ahkConvert(i, "auto") === o));

  // ---- cheap structural sanity for a file we can't execute here
  const code = src.replace(/;.*$/gm, "").replace(/"(?:[^"`]|`[\s\S]|"")*"/g, '""');
  const count_ = (re_) => (code.match(re_) || []).length;
  t("ahk: braces balanced", count_(/\{/g) === count_(/\}/g));
  t("ahk: parentheses balanced", count_(/\(/g) === count_(/\)/g));
  t("ahk: requires AutoHotkey v2", /^#Requires AutoHotkey v2\.0/m.test(src));
  t("ahk: icon file exists", fs.existsSync(path.join(root, "desktop", "windows", "baddel.ico")));
};

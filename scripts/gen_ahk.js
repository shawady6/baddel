#!/usr/bin/env node
// Generates the conversion tables inside desktop/windows/baddel.ahk from
// extension/shared.js, so the Windows app and the extension can never drift apart.
//   node scripts/gen_ahk.js
const fs = require("fs");
const path = require("path");

const root = path.join(__dirname, "..");
const A = require(path.join(root, "extension", "shared.js"));
const file = path.join(root, "desktop", "windows", "baddel.ahk");

// AutoHotkey v2 string escapes: ` is the escape char, " is doubled
const esc = (s) => s.replace(/`/g, "``").replace(/"/g, '""');

function tables() {
  const en = [];
  const ar = new Map();
  for (let code = 33; code <= 126; code++) {
    const c = String.fromCharCode(code);
    const r = A.convert(c, "en2ar");
    if (r === c) continue;
    en.push([c, r]);
    if (r.length === 1 && /[؀-ۿ]/.test(r) && !ar.has(r)) ar.set(r, A.convert(r, "ar2en"));
  }
  return { en, ar: [...ar] };
}

function build() {
  const { en, ar } = tables();
  const lines = [];
  for (const [k, v] of en) lines.push(`enToAr["${esc(k)}"] := "${esc(v)}"`);
  for (const [k, v] of ar) lines.push(`arToEn["${esc(k)}"] := "${esc(v)}"`);
  return lines.join("\n");
}

if (require.main === module) {
  const src = fs.readFileSync(file, "utf8").replace(/^﻿/, "");
  const re = /(;@@MAPS_BEGIN\r?\n)[\s\S]*?(\r?\n?;@@MAPS_END)/;
  if (!re.test(src)) throw new Error("markers not found in baddel.ahk");
  const out = src.replace(re, (_, a, b) => a + build() + "\n" + b.replace(/^\r?\n/, ""));
  // UTF-8 with BOM so every AutoHotkey version reads the Arabic letters correctly
  fs.writeFileSync(file, "﻿" + out.replace(/\r\n/g, "\n"), "utf8");
  console.log("baddel.ahk tables updated:", build().split("\n").length, "entries");
}

module.exports = { build, tables, file };

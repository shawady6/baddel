// Run: node tests/test_logic.js
const fs = require("fs");
const path = require("path");

const root = path.join(__dirname, "..");
const extDir = path.join(root, "extension");
const manifest = JSON.parse(fs.readFileSync(path.join(extDir, "manifest.json"), "utf8"));

let fail = 0;
const t = (name, ok) => { if (!ok) fail++; console.log(ok ? "PASS" : "FAIL", name); };

// every file the manifest points at must exist
const refs = [
  ...Object.values(manifest.icons), ...Object.values(manifest.action.default_icon),
  manifest.action.default_popup, manifest.options_ui.page, manifest.background.service_worker,
  ...manifest.content_scripts.flatMap((c) => c.js)
];
t("manifest references exist", refs.every((r) => fs.existsSync(path.join(extDir, r))));
t("manifest is MV3", manifest.manifest_version === 3);

// the website copy must match the extension copy
t("docs/shared.js is in sync", fs.readFileSync(path.join(root, "docs/shared.js"), "utf8") ===
  fs.readFileSync(path.join(extDir, "shared.js"), "utf8"));

const A = require(path.join(extDir, "shared.js"));

t("hgsghl -> السلام", A.convert("hgsghl") === "السلام");
t("السلام -> hgsghl", A.convert("السلام") === "hgsghl");
t("lvpfh -> مرحبا", A.convert("lvpfh") === "مرحبا");
t("hgsghl ugd;l -> السلام عليكم", A.convert("hgsghl ugd;l") === "السلام عليكم");
t("round trip", A.convert(A.convert("lvpfh hgsghl")) === "lvpfh hgsghl");
t("لا -> gh", A.convert("لا") === "gh");
t("digits untouched", A.convert("123") === "123");
t("forced ar2en leaves latin alone", A.convert("abc", "ar2en") === "abc");
t("forced en2ar", A.convert("hgsghl", "en2ar") === "السلام");

const key = (o) => ({ code: "KeyX", ctrlKey: true, altKey: false, shiftKey: true, metaKey: false, ...o });
const sc = A.DEFAULTS.shortcut;
t("default shortcut matches", A.matchesShortcut(key({}), sc));
t("layout independent", A.matchesShortcut({ ...key({}), key: "س" }, sc));
t("no match without shift", !A.matchesShortcut(key({ shiftKey: false }), sc));
t("no match with extra alt", !A.matchesShortcut(key({ altKey: true }), sc));
t("format Ctrl+Shift+X", JSON.stringify(A.formatShortcut(sc)) === '["Ctrl","Shift","X"]');

const v = (o) => A.validateShortcutEvent({ code: "KeyK", ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...o });
t("Escape cancels", A.validateShortcutEvent({ code: "Escape" }).status === "cancel");
t("modifier alone pending", v({ code: "ControlLeft", ctrlKey: true }).status === "pending");
t("letter alone rejected", v({}).error === "no-modifier");
t("shift only rejected", v({ shiftKey: true }).error === "no-modifier");
t("Ctrl+C reserved", v({ code: "KeyC", ctrlKey: true }).error === "reserved");
t("Ctrl+Shift+T reserved", v({ code: "KeyT", ctrlKey: true, shiftKey: true }).error === "reserved");
const ok = v({ code: "Digit7", altKey: true });
t("Alt+7 accepted", ok.status === "ok" && ok.shortcut.code === "Digit7");

t("normalize garbage -> defaults",
  JSON.stringify(A.normalizeSettings({ shortcut: 5, direction: "x" })) === JSON.stringify(A.normalizeSettings(null)));
t("normalize keeps disabled", A.normalizeSettings({ enabled: false }).enabled === false);

process.exit(fail ? 1 : 0);

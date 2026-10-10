(() => {
  // [US-layout key, what the Arabic 101 layout types on that same physical key]
  const BASE = [
    ["`","ذ"],["q","ض"],["w","ص"],["e","ث"],["r","ق"],["t","ف"],["y","غ"],["u","ع"],["i","ه"],["o","خ"],["p","ح"],["[","ج"],["]","د"],
    ["a","ش"],["s","س"],["d","ي"],["f","ب"],["g","ل"],["h","ا"],["j","ت"],["k","ن"],["l","م"],[";","ك"],["'","ط"],
    ["z","ئ"],["x","ء"],["c","ؤ"],["v","ر"],["b","لا"],["n","ى"],["m","ة"],[",","و"],[".","ز"],["/","ظ"]
  ];
  const SHIFT = [
    ["~","ّ"],["Q","َ"],["W","ً"],["E","ُ"],["R","ٌ"],["T","لإ"],["Y","إ"],["U","‘"],["I","÷"],["O","×"],["P","؛"],
    ["A","ِ"],["S","ٍ"],["D","]"],["F","["],["G","لأ"],["H","أ"],["J","ـ"],["K","،"],["L","/"],
    ["Z","~"],["X","ْ"],["C","{"],["V","}"],["B","لآ"],["N","آ"],["M","’"],["?","؟"]
  ];

  const enToAr = Object.create(null);
  const arToEn = Object.create(null);
  for (const [en, ar] of [...BASE, ...SHIFT]) {
    if (!(en in enToAr)) enToAr[en] = ar;
    // Arabic -> English: single Arabic-block characters only. Lam-alef ligature keys
    // are skipped on purpose: "لا" is far more often ل + ا typed as two keys (g + h).
    if (ar.length === 1 && /[؀-ۿ]/.test(ar) && !(ar in arToEn)) arToEn[ar] = en;
  }

  const mapText = (text, map) => {
    let out = "";
    for (let i = 0; i < text.length; i++) {
      const pair = text.slice(i, i + 2);
      if (pair.length === 2 && pair in map) { out += map[pair]; i++; continue; }
      const ch = text[i];
      out += ch in map ? map[ch] : ch;
    }
    return out;
  };

  // mode: "auto" (decide from the dominant script), "ar2en" or "en2ar".
  const convert = (text, mode = "auto") => {
    if (mode === "ar2en") return mapText(text, arToEn);
    if (mode === "en2ar") return mapText(text, enToAr);
    const arabic = (text.match(/[؀-ۿ]/g) || []).length;
    const latin = (text.match(/[A-Za-z]/g) || []).length;
    if (!arabic && !latin) return text;
    return mapText(text, arabic >= latin ? arToEn : enToAr);
  };

  // ---------- settings ----------
  const DEFAULTS = {
    enabled: true,
    lastWord: true,
    direction: "auto",
    shortcut: { code: "KeyX", ctrl: true, alt: false, shift: true, meta: false }
  };

  const normalizeShortcut = (sc) => {
    if (!sc || typeof sc.code !== "string" || !sc.code) return { ...DEFAULTS.shortcut };
    return { code: sc.code, ctrl: !!sc.ctrl, alt: !!sc.alt, shift: !!sc.shift, meta: !!sc.meta };
  };

  const normalizeSettings = (s) => {
    s = s || {};
    return {
      enabled: s.enabled !== false,
      lastWord: s.lastWord !== false,
      direction: ["auto", "ar2en", "en2ar"].includes(s.direction) ? s.direction : "auto",
      shortcut: normalizeShortcut(s.shortcut)
    };
  };

  // ---------- shortcuts ----------
  const isMac = typeof navigator !== "undefined" && /Mac/i.test(navigator.platform || "");

  const matchesShortcut = (e, sc) =>
    e.code === sc.code &&
    !!e.ctrlKey === sc.ctrl && !!e.altKey === sc.alt &&
    !!e.shiftKey === sc.shift && !!e.metaKey === sc.meta;

  const SPECIAL = {
    Space: "Space", Slash: "/", Backslash: "\\", Comma: ",", Period: ".", Semicolon: ";",
    Quote: "'", BracketLeft: "[", BracketRight: "]", Backquote: "`", Minus: "-", Equal: "=",
    ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→"
  };

  const codeLabel = (code) => {
    if (SPECIAL[code]) return SPECIAL[code];
    if (/^Key[A-Z]$/.test(code)) return code.slice(3);
    if (/^Digit\d$/.test(code)) return code.slice(5);
    if (/^Numpad/.test(code)) return "Num " + code.slice(6);
    return code;
  };

  // -> ["Ctrl", "Shift", "X"]
  const formatShortcut = (sc) => {
    const parts = [];
    if (sc.ctrl) parts.push(isMac ? "⌃" : "Ctrl");
    if (sc.alt) parts.push(isMac ? "⌥" : "Alt");
    if (sc.shift) parts.push(isMac ? "⇧" : "Shift");
    if (sc.meta) parts.push(isMac ? "⌘" : "Win");
    parts.push(codeLabel(sc.code));
    return parts;
  };

  const idOf = (sc) =>
    [sc.ctrl && "ctrl", sc.alt && "alt", sc.shift && "shift", sc.meta && "meta", sc.code]
      .filter(Boolean).join("+");

  // Combos the browser or the OS keeps for itself, plus basic editing keys.
  const RESERVED = new Set([
    "ctrl+KeyC", "ctrl+KeyV", "ctrl+KeyX", "ctrl+KeyZ", "ctrl+KeyY", "ctrl+KeyA",
    "ctrl+KeyT", "ctrl+KeyW", "ctrl+KeyN", "ctrl+shift+KeyT", "ctrl+shift+KeyN", "ctrl+shift+KeyW",
    "ctrl+Tab", "ctrl+shift+Tab", "alt+F4",
    "meta+KeyC", "meta+KeyV", "meta+KeyX", "meta+KeyZ", "meta+KeyA", "meta+KeyT", "meta+KeyW",
    "meta+KeyN", "meta+KeyQ", "meta+shift+KeyT", "meta+shift+KeyN", "meta+shift+KeyW"
  ]);

  const MOD_CODE = /^(Control|Shift|Alt|Meta|OS)(Left|Right)?$/;

  // Used by the recorder. status: cancel | pending | error | ok
  const validateShortcutEvent = (e) => {
    if (e.code === "Escape") return { status: "cancel" };
    if (MOD_CODE.test(e.code)) return { status: "pending" };
    const sc = { code: e.code, ctrl: !!e.ctrlKey, alt: !!e.altKey, shift: !!e.shiftKey, meta: !!e.metaKey };
    if (!(sc.ctrl || sc.alt || sc.meta)) return { status: "error", error: "no-modifier" };
    if (RESERVED.has(idOf(sc))) return { status: "error", error: "reserved" };
    return { status: "ok", shortcut: sc };
  };

  const api = {
    convert, DEFAULTS, normalizeSettings, matchesShortcut,
    formatShortcut, validateShortcutEvent, codeLabel
  };

  if (typeof module !== "undefined" && module.exports) module.exports = api;
  globalThis.ATF = api;
})();

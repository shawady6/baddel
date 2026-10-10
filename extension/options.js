const { convert, DEFAULTS, normalizeSettings, formatShortcut, validateShortcutEvent } = globalThis.ATF;

const $ = (id) => document.getElementById(id);
const els = {
  enabled: $("enabled"), lastWord: $("lastWord"), keys: $("keys"), recorder: $("recorder"),
  hint: $("rec-hint"), msg: $("shortcut-msg"), reset: $("reset"), toast: $("toast"),
  tryIn: $("try-in"), tryOut: $("try-out"), ver: $("ver")
};

const ERRORS = {
  "no-modifier": "لازم تضغط Ctrl أو Alt أو ⌘ مع الحرف.",
  reserved: "الاختصار ده محجوز للمتصفح أو للنسخ واللصق، جرّب غيره."
};

let settings = normalizeSettings(null);
let recording = false;
let toastTimer;

function showToast(text) {
  els.toast.textContent = text;
  els.toast.classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => els.toast.classList.remove("show"), 1600);
}

function setMessage(text, ok = false) {
  els.msg.textContent = text;
  els.msg.classList.toggle("ok", ok);
}

function renderKeys(parts) {
  els.keys.replaceChildren(...parts.map((p) => {
    const k = document.createElement("kbd");
    k.textContent = p;
    return k;
  }));
}

function renderTry() {
  els.tryOut.value = convert(els.tryIn.value, settings.direction);
}

function render() {
  els.enabled.checked = settings.enabled;
  els.lastWord.checked = settings.lastWord;
  document.querySelectorAll('input[name="direction"]').forEach((r) => {
    r.checked = r.value === settings.direction;
  });
  if (!recording) renderKeys(formatShortcut(settings.shortcut));
  renderTry();
}

async function save(patch, message = "اتحفظت التغييرات ✓") {
  settings = { ...settings, ...patch };
  render();
  try {
    await chrome.storage.sync.set(settings);
    showToast(message);
  } catch (e) {
    showToast("مقدرتش أحفظ الإعدادات");
  }
}

// ---------- shortcut recorder ----------
function startRecording() {
  recording = true;
  els.recorder.classList.add("recording");
  els.hint.textContent = "اضغط الاختصار الجديد… (Esc للإلغاء)";
  setMessage("");
  els.keys.replaceChildren();
}

function stopRecording() {
  recording = false;
  els.recorder.classList.remove("recording");
  els.hint.textContent = "اضغط للتغيير";
  renderKeys(formatShortcut(settings.shortcut));
}

document.addEventListener("keydown", (e) => {
  if (!recording) return;
  e.preventDefault();
  e.stopPropagation();

  const result = validateShortcutEvent(e);
  if (result.status === "cancel") { setMessage(""); stopRecording(); return; }
  if (result.status === "pending") {
    // Live preview of the modifiers being held.
    const held = [];
    if (e.ctrlKey) held.push("Ctrl");
    if (e.altKey) held.push("Alt");
    if (e.shiftKey) held.push("Shift");
    if (e.metaKey) held.push("⌘");
    renderKeys(held);
    return;
  }
  if (result.status === "error") { setMessage(ERRORS[result.error]); return; }

  stopRecording();
  setMessage("");
  save({ shortcut: result.shortcut }, "الاختصار الجديد اتحفظ ✓");
}, true);

els.recorder.addEventListener("click", () => (recording ? stopRecording() : startRecording()));
els.recorder.addEventListener("blur", () => { if (recording) stopRecording(); });
els.reset.addEventListener("click", () => {
  if (recording) stopRecording();
  setMessage("");
  save({ shortcut: { ...DEFAULTS.shortcut } }, "رجع للاختصار الافتراضي ✓");
});

// ---------- other controls ----------
els.enabled.addEventListener("change", () => save({ enabled: els.enabled.checked }));
els.lastWord.addEventListener("change", () => save({ lastWord: els.lastWord.checked }));
document.querySelectorAll('input[name="direction"]').forEach((r) =>
  r.addEventListener("change", () => r.checked && save({ direction: r.value })));
els.tryIn.addEventListener("input", renderTry);

els.ver.textContent = "v" + chrome.runtime.getManifest().version;

chrome.storage.sync.get(null).then((stored) => {
  settings = normalizeSettings(stored);
  render();
});
chrome.storage.onChanged.addListener(() => {
  chrome.storage.sync.get(null).then((stored) => { settings = normalizeSettings(stored); render(); });
});

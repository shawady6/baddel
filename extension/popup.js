const { normalizeSettings, formatShortcut } = globalThis.ATF;

const enabledEl = document.getElementById("enabled");
const keysEl = document.getElementById("keys");
const stateEl = document.getElementById("state-text");

let settings = normalizeSettings(null);

function render() {
  enabledEl.checked = settings.enabled;
  stateEl.textContent = settings.enabled ? "شغّالة" : "متوقفة";
  keysEl.replaceChildren(...formatShortcut(settings.shortcut).map((p) => {
    const k = document.createElement("kbd");
    k.textContent = p;
    return k;
  }));
}

enabledEl.addEventListener("change", async () => {
  settings = { ...settings, enabled: enabledEl.checked };
  render();
  try { await chrome.storage.sync.set(settings); } catch (e) { /* keep UI state */ }
});

document.getElementById("open").addEventListener("click", () => {
  chrome.runtime.openOptionsPage();
  window.close();
});

chrome.storage.sync.get(null).then((stored) => {
  settings = normalizeSettings(stored);
  render();
});

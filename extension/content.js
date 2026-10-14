(() => {
  if (window.__atfLoaded) return;
  window.__atfLoaded = true;

  const { convert, normalizeSettings, matchesShortcut } = globalThis.ATF;

  let settings = normalizeSettings(null);
  const loadSettings = () => {
    try {
      chrome.storage.sync.get(null, (stored) => {
        if (chrome.runtime.lastError) return;
        settings = normalizeSettings(stored);
      });
    } catch (e) { /* extension was reloaded; this script is now orphaned */ }
  };
  loadSettings();
  try { chrome.storage.onChanged.addListener(loadSettings); } catch (e) { /* orphaned */ }

  function fixInput(el) {
    let start, end;
    try { start = el.selectionStart; end = el.selectionEnd; } catch (e) { return; }
    if (start == null || end == null) return;
    const value = el.value;

    let from = start, to = end;
    if (start === end) {
      if (!settings.lastWord) return;
      // No selection: take the last token before the caret (ignoring trailing spaces).
      const m = value.slice(0, start).match(/(\S+)(\s*)$/u);
      if (!m) return;
      to = start - m[2].length;
      from = to - m[1].length;
    }
    const original = value.slice(from, to);
    const changed = convert(original, settings.direction);
    if (changed === original) return;

    el.setSelectionRange(from, to);
    // execCommand keeps undo history and fires the events frameworks listen to.
    if (!document.execCommand("insertText", false, changed)) {
      el.setRangeText(changed, from, to, "end");
      el.dispatchEvent(new Event("input", { bubbles: true }));
    }
    if (start === end) {
      const caret = start + (changed.length - original.length);
      el.setSelectionRange(caret, caret);
    } else {
      // Keep the result selected so pressing the shortcut again toggles it back.
      el.setSelectionRange(from, from + changed.length);
    }
  }

  // Rich-text selections can span paragraphs, list items and bold/italic runs.
  // range.toString() drops those boundaries and insertText of the joined string would
  // flatten them, so convert each text node on its own and keep the structure.
  function fixEditableRange(range, sel) {
    const root = range.commonAncestorContainer;
    const parts = [];
    if (root.nodeType === Node.TEXT_NODE) {
      parts.push({ node: root, s: range.startOffset, e: range.endOffset });
    } else {
      const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (!range.intersectsNode(n)) continue;
        const s = n === range.startContainer ? range.startOffset : 0;
        const e = n === range.endContainer ? range.endOffset : n.length;
        if (e > s) parts.push({ node: n, s, e });
      }
    }
    if (!parts.length) return;
    const whole = parts.map((x) => x.node.data.slice(x.s, x.e)).join("\n");
    let mode = settings.direction;
    if (mode === "auto") {
      const ar = (whole.match(/[\u0600-\u06FF]/g) || []).length;
      const la = (whole.match(/[A-Za-z]/g) || []).length;
      if (!ar && !la) return;
      mode = ar >= la ? "ar2en" : "en2ar";
    }
    let any = false;
    for (const x of parts) {
      const original = x.node.data.slice(x.s, x.e);
      const changed = convert(original, mode);
      x.rn = x.node; x.rs = x.s; x.re = x.e;
      if (changed === original) continue;
      any = true;
      sel.setBaseAndExtent(x.node, x.s, x.node, x.e);
      if (document.execCommand("insertText", false, changed) && sel.focusNode && sel.focusNode.nodeType === Node.TEXT_NODE) {
        // the editor may have swapped the node: remember where the result really is
        x.rn = sel.focusNode; x.re = sel.focusOffset; x.rs = x.re - changed.length;
      } else {
        x.node.replaceData(x.s, x.e - x.s, changed);
        x.rn = x.node; x.rs = x.s; x.re = x.s + changed.length;
      }
    }
    if (!any) return;
    try {
      // keep the result selected so pressing the shortcut again toggles it back
      const first = parts[0], last = parts[parts.length - 1];
      if (first.rn.isConnected && last.rn.isConnected)
        sel.setBaseAndExtent(first.rn, Math.max(first.rs, 0), last.rn, last.re);
    } catch (e) { /* selection restore is best-effort */ }
  }

  function fixEditable() {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) return;
    const range = sel.getRangeAt(0).cloneRange();
    const wasCollapsed = range.collapsed;
    if (!wasCollapsed) return fixEditableRange(range, sel);
    let trailing = 0;

    if (wasCollapsed) {
      if (!settings.lastWord) return;
      const node = range.startContainer;
      if (node.nodeType !== Node.TEXT_NODE) return;
      const m = node.textContent.slice(0, range.startOffset).match(/(\S+)(\s*)$/u);
      if (!m) return;
      trailing = m[2].length;
      const endOffset = range.startOffset - trailing;
      range.setStart(node, endOffset - m[1].length);
      range.setEnd(node, endOffset);
    }

    const original = range.toString();
    const changed = convert(original, settings.direction);
    if (changed === original) return;

    sel.removeAllRanges();
    sel.addRange(range);
    if (!document.execCommand("insertText", false, changed)) {
      range.deleteContents();
      range.insertNode(document.createTextNode(changed));
    }

    try {
      const node = sel.focusNode;
      const off = sel.focusOffset;
      if (node && node.nodeType === Node.TEXT_NODE) {
        if (wasCollapsed) {
          sel.collapse(node, Math.min(off + trailing, node.textContent.length));
        } else if (off >= changed.length) {
          const r = document.createRange();
          r.setStart(node, off - changed.length);
          r.setEnd(node, off);
          sel.removeAllRanges();
          sel.addRange(r);
        }
      }
    } catch (e) { /* caret placement is best-effort */ }
  }

  function onKeyDown(e) {
    // After an extension reload the old copy of this script loses chrome.runtime.id:
    // stop it so the new copy is the only one reacting (otherwise it would toggle twice).
    if (!chrome.runtime?.id) {
      document.removeEventListener("keydown", onKeyDown, true);
      return;
    }
    if (!settings.enabled || e.repeat || e.isComposing) return;
    if (!matchesShortcut(e, settings.shortcut)) return;

    const el = document.activeElement;
    if (!el) return;
    const isField = el.tagName === "TEXTAREA" ||
      (el.tagName === "INPUT" && /^(text|search|url|tel)$/i.test(el.type));
    if (isField) {
      e.preventDefault(); e.stopPropagation();
      fixInput(el);
    } else if (el.isContentEditable) {
      e.preventDefault(); e.stopPropagation();
      fixEditable();
    }
  }

  // Capture phase + physical key codes: works whatever the current keyboard language is.
  document.addEventListener("keydown", onKeyDown, true);
})();

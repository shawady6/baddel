(() => {
  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => [...r.querySelectorAll(s)];
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  // English strings, keyed by data-i. Arabic is the HTML itself.
  const EN = {
    nav_demo: "Try it", nav_install: "Install", nav_faq: "FAQ",
    pill: "Free · Open source · Runs inside your browser",
    h1: "Typed Arabic on an English keyboard? <em>Baddel</em> it in one keypress.",
    lead: "<b>Baddel · بدّل</b> is a Chrome &amp; Edge extension that converts text typed in the wrong keyboard language (like <code dir=\"ltr\">hgsghl</code> instead of «السلام») with a single shortcut — no deleting and retyping.",
    dl: "Download extension (ZIP)", how: "How do I install it?",
    platforms: "Desktop only: Chrome, Edge, Brave and Opera.",
    mobile_note: "The extension only works in desktop browsers. Open this page on your computer to install it.",
    demo_h: "Try it now",
    demo_sub: "Type or paste text in the wrong language and see the result instantly. This is the same conversion map used inside the extension.",
    in_label: "Text typed in the wrong language", out_label: "Result", chips_label: "Try an example:",
    f_h: "Why Baddel?",
    f1_t: "Works whatever your keyboard language is",
    f1_d: "The shortcut uses the physical key, so it never gets confused whether you're on Arabic or English.",
    f2_t: "Convert and undo with the same press",
    f2_d: "The result stays selected, and pressing the shortcut again flips it back.",
    f3_t: "Your shortcut, your way",
    f3_d: "A settings page with a shortcut recorder, conversion direction, and an on/off switch.",
    f4_t: "Fully private",
    f4_d: "Everything happens inside the browser: no network requests and no data collection.",
    inst_h: "Install in 6 steps",
    inst_sub: "The extension isn't on the Chrome Web Store yet, so you install it manually. It takes about two minutes.",
    st1_t: "Download the file",
    st1_d: "Click “Download extension” to get <code dir=\"ltr\">baddel.zip</code>.",
    dl2: "Download baddel.zip",
    st2_t: "Extract it to a permanent place",
    st2_d: "Right-click → Extract all. Keep the <b>baddel</b> folder somewhere you won't delete, because the browser loads the extension from it.",
    st3_t: "Open the extensions page",
    st3_d: "Copy the address and paste it into the address bar (these links can't be opened by clicking):",
    st4_t: "Turn on Developer mode",
    st4_d: "Switch on <b>Developer mode</b> at the top right.",
    st5_t: "Load the folder",
    st5_d: "Click <b>Load unpacked</b> and choose the <b>baddel</b> folder (the one containing <code dir=\"ltr\">manifest.json</code>).",
    st6_t: "Try it",
    st6_d: "Pin the icon from the puzzle 🧩 menu. Then type in any field, select the text and press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>X</kbd>.",
    warn_t: "Warning when opening the browser",
    warn_d: "Chrome may show a notice about developer-mode extensions when it starts. That's normal for manually installed extensions; ignore it or press Cancel.",
    upd_t: "Updating",
    upd_d: "When a new release comes out: download it, extract it over the same folder, then press the reload ↻ button on the extension card.",
    faq_h: "FAQ",
    q1: "Why does it ask for access to all sites?",
    a1: "So it can read the text in the field you're typing in and convert it on any website. It does nothing until you press the shortcut, and it only touches the text you selected or the last word.",
    q2: "Does it send my text anywhere?",
    a2: "No. The code makes no network requests, and only your settings are stored in the browser. It's open source, so you can check it yourself on GitHub.",
    q3: "Why isn't it on the Chrome Web Store?",
    a3: "It isn't published there yet. Manual installation gives you exactly the same extension, and store links will be added when it's available.",
    q4: "Where does it work?",
    a4: "In regular text fields (input and textarea) and most editors. It doesn't work on <bdi dir=\"ltr\">chrome://</bdi> pages, canvas editors like Google Docs, or email and password fields.",
    q7: "Why doesn't it work in the browser's address bar?",
    a7: "The address bar is part of the browser's own interface, not a web page, and browsers don't let extensions touch it for security reasons, so there's no way to make it work there. Workaround: type into any field on a page (such as the Google search box in a new tab), or convert the text in another field and then cut and paste it into the address bar.",
    q5: "How do I change the shortcut?",
    a5: "Click the extension icon → <b>Open settings</b> → click the shortcut box and press the new combination.",
    q6: "Is there a Firefox or mobile version?",
    a6: "Not right now. Chrome on mobile doesn't support extensions, and a Firefox version isn't ready yet.",
    foot_by: "Made by Ibrahim · MIT license", foot_privacy: "Privacy policy", foot_issues: "Report an issue"
  };

  const STR = {
    copy: { ar: "نسخ", en: "Copy" },
    copied: { ar: "اتنسخ ✓", en: "Copied ✓" }
  };

  let lang = "ar";
  const i18n = $$("[data-i]");
  i18n.forEach((el) => { el.dataset.ar = el.innerHTML; });
  const titles = {
    ar: document.title,
    en: "Baddel – بدّل | Arabic ↔ English keyboard fixer"
  };

  const copyBtn = $("#copy-url");
  function setLang(l) {
    lang = l;
    document.documentElement.lang = l;
    document.documentElement.dir = l === "ar" ? "rtl" : "ltr";
    i18n.forEach((el) => {
      el.innerHTML = l === "ar" ? el.dataset.ar : (EN[el.dataset.i] ?? el.dataset.ar);
    });
    $("#lang").textContent = l === "ar" ? "EN" : "عربي";
    copyBtn.textContent = STR.copy[l];
    document.title = titles[l];
    try { localStorage.setItem("baddel-lang", l); } catch (e) { /* storage may be blocked */ }
  }
  $("#lang").addEventListener("click", () => setLang(lang === "ar" ? "en" : "ar"));

  // ---------- live demo ----------
  const inEl = $("#in"), outEl = $("#out");
  const render = () => { outEl.value = ATF.convert(inEl.value); };
  inEl.addEventListener("input", render);
  $$(".chip").forEach((c) => c.addEventListener("click", () => {
    inEl.value = c.dataset.sample; render(); inEl.focus();
  }));
  inEl.value = "hgsghl ugd;l"; render();

  // ---------- hero animation ----------
  (function hero() {
    const el = $("#typed");
    const keys = $$(".keys-row kbd");
    const wrong = "hgsghl ugd;l";
    const right = ATF.convert(wrong);
    if (reduceMotion) { el.textContent = right; el.classList.add("fixed"); return; }
    const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
    (async function loop() {
      for (;;) {
        el.classList.remove("fixed"); el.textContent = "";
        for (let i = 1; i <= wrong.length; i++) { el.textContent = wrong.slice(0, i); await sleep(90); }
        await sleep(700);
        keys.forEach((k) => k.classList.add("pressed"));
        await sleep(350);
        el.textContent = right; el.classList.add("fixed");
        keys.forEach((k) => k.classList.remove("pressed"));
        await sleep(2600);
      }
    })();
  })();

  // ---------- install helper ----------
  function detectBrowser() {
    const ua = navigator.userAgent;
    if (/Edg\//.test(ua)) return "edge";
    if (navigator.brave) return "brave";
    if (/OPR\//.test(ua)) return "opera";
    return "chrome";
  }
  const tabs = $$(".tabs button");
  const urlEl = $("#ext-url");
  function selectBrowser(b) {
    tabs.forEach((t) => {
      const on = t.dataset.b === b;
      t.setAttribute("aria-selected", on ? "true" : "false");
      if (on) urlEl.textContent = t.dataset.url;
    });
  }
  tabs.forEach((t) => t.addEventListener("click", () => selectBrowser(t.dataset.b)));
  selectBrowser(detectBrowser());

  async function copyText(text) {
    try { await navigator.clipboard.writeText(text); return true; } catch (e) {
      const ta = document.createElement("textarea");
      ta.value = text; ta.style.cssText = "position:fixed;opacity:0";
      document.body.appendChild(ta); ta.select();
      let ok = false;
      try { ok = document.execCommand("copy"); } catch (e2) { /* ignore */ }
      ta.remove();
      return ok;
    }
  }
  let copyTimer;
  copyBtn.addEventListener("click", async () => {
    if (await copyText(urlEl.textContent)) {
      copyBtn.textContent = STR.copied[lang];
      clearTimeout(copyTimer);
      copyTimer = setTimeout(() => { copyBtn.textContent = STR.copy[lang]; }, 1600);
    }
  });

  if (/Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent)) $("#mobile").hidden = false;

  let saved = null;
  try { saved = localStorage.getItem("baddel-lang"); } catch (e) { /* ignore */ }
  setLang(saved === "en" ? "en" : "ar");
})();

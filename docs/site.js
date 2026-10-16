(() => {
  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => [...r.querySelectorAll(s)];
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  // English strings, keyed by data-i. Arabic is the HTML itself.
  const EN = {
    nav_demo: "Try it", nav_install: "Download", nav_faq: "FAQ",
    pill: "Free · Open source · Runs inside your browser",
    h1: "Typed Arabic on an English keyboard? <em>Baddel</em> it in one keypress.",
    lead: "Type as usual. If the text comes out in the wrong language (like <code dir=\"ltr\">hgsghl</code> instead of «السلام»), select it and press one shortcut to fix it.",
    dl: "Download extension (ZIP)", how: "How do I install it?",
    dl_win: "Download Windows app",
    platforms: "Extension for Chrome and Edge; the Windows app works in any program.",
    mobile_note: "The extension only works in desktop browsers. Open this page on your computer to install it.",
    demo_h: "Try it now",
    demo_sub: "Type or paste text in the wrong language and see the result instantly.",
    in_label: "Text typed in the wrong language", out_label: "Result", chips_label: "Try an example:",
    f1_t: "Works with any keyboard language", f1_d: "One shortcut, whether you're on Arabic or English.",
    f2_t: "Undo built in", f2_d: "Press the shortcut again to put the text back.",
    f4_t: "Fully private", f4_d: "Everything happens on your device. No internet, no data collection.",
    inst_h: "Download and go", inst_sub: "Pick what suits you. Both are free and open source.",
    d_h: "Windows app", d_sub: "Works in any program, even the address bar.", d_dl: "Download installer", d_dlp: "Portable",
    d1_d: "Run <code dir=\"ltr\">Baddel-Setup.exe</code> and click Next.",
    d2_d: "If Windows shows a warning: <b>More info → Run anyway</b>.",
    d3_d: "Select the wrong text and press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>X</kbd>. Double-click the tray icon for settings.",
    e_h: "Browser extension", e_sub: "Chrome, Edge, Brave and Opera.", dl2: "Download (ZIP)",
    e1_d: "Extract it to a permanent folder.",
    e2_d: "Open the extensions page, turn on <b>Developer mode</b> → <b>Load unpacked</b> → choose the folder.",
    faq_h: "Quick questions",
    q2: "Does it send my text anywhere?", a2: "No. There's no internet access at all, and the code is open on GitHub for you to check.",
    q1: "Why does the extension ask for all sites?", a1: "So it can convert on any website. It does nothing until you press the shortcut.",
    q7: "Why doesn't the extension work in the address bar?", a7: "Browsers don't let extensions touch it. Use the <a href=\"#desktop\">Windows app</a>, which does.",
    q5: "How do I change the shortcut?", a5: "From the extension's settings, or double-click the Windows app's tray icon.",
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

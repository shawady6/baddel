(() => {
  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => [...r.querySelectorAll(s)];
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  // English strings, keyed by data-i. Arabic is the HTML itself.
  const EN = {
    nav_demo: "Try it", nav_install: "Download", nav_faq: "FAQ",
    pill: "Free · Open source · Runs on your computer",
    h1: "Typed Arabic on an English keyboard? <em>Baddel</em> it in one keypress.",
    lead: "Type as usual. If the text comes out in the wrong language (like <code dir=\"ltr\">hgsghl</code> instead of «السلام»), press one shortcut to fix it, in any program.",
    dl_win: "Download for Windows",
    platforms: "Windows 10 and 11. Small, fast, no internet needed.",
    notwin: "The app runs on Windows only. Open this page on a Windows computer to download it.",
    demo_h: "Try it now",
    demo_sub: "Type or paste text in the wrong language and see the result instantly.",
    in_label: "Text typed in the wrong language", out_label: "Result", chips_label: "Try an example:",
    f1_t: "Works in any program", f1_d: "Word, browsers, chat apps, even the address bar.",
    f2_t: "Undo built in", f2_d: "Press the shortcut again to put the text back.",
    f4_t: "Fully private", f4_d: "Everything happens on your device. No internet, no data collection.",
    inst_h: "Download and go", inst_sub: "Free and open source.",
    d_h: "Windows app", d_sub: "Works in any program, even the address bar.", d_dl: "Download installer", d_portable: "or the portable version",
    d1_d: "Run <code dir=\"ltr\">Baddel-Setup.exe</code> and click Next.",
    d2_d: "If Windows shows a warning: <b>More info → Run anyway</b>.",
    d3_d: "Select the wrong text and press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>X</kbd>. Double-click the tray icon for settings.",
    faq_h: "Quick questions",
    q2: "Does it send my text anywhere?", a2: "No. There's no internet access at all, and the code is open on GitHub for you to check.",
    q1: "Why does Windows show a warning?", a1: "The app isn't code-signed yet. Click <b>More info → Run anyway</b>. Some antivirus tools distrust any program that simulates keystrokes; the code is open for review.",
    q7: "Does it work with nothing selected?", a7: "Yes. It converts the last word, or the last words you typed in the same language. Press again to undo.",
    q5: "How do I change the shortcut?", a5: "Double-click the tray icon, click the shortcut box and press the keys you want.",
    foot_by: "Made by Ibrahim · MIT license", foot_privacy: "Privacy policy", foot_issues: "Report an issue"
  };

    let lang = "ar";
  const i18n = $$("[data-i]");
  i18n.forEach((el) => { el.dataset.ar = el.innerHTML; });
  const titles = {
    ar: document.title,
    en: "Baddel – بدّل | Arabic ↔ English keyboard fixer"
  };

  function setLang(l) {
    lang = l;
    document.documentElement.lang = l;
    document.documentElement.dir = l === "ar" ? "rtl" : "ltr";
    i18n.forEach((el) => {
      el.innerHTML = l === "ar" ? el.dataset.ar : (EN[el.dataset.i] ?? el.dataset.ar);
    });
    $("#lang").textContent = l === "ar" ? "EN" : "عربي";
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

  if (!/Windows/i.test(navigator.userAgent)) $("#notwin").hidden = false;

  let saved = null;
  try { saved = localStorage.getItem("baddel-lang"); } catch (e) { /* ignore */ }
  setLang(saved === "en" ? "en" : "ar");
})();

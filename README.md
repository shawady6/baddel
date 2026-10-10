<div align="center">

<img src="extension/icons/icon-128.png" alt="Baddel" width="96">

# Baddel · بدّل

**Typed Arabic on an English keyboard (or the opposite)? Fix it with one shortcut.**
**كتبت بلغة غلط؟ بدّل بضغطة.**

[**Website / الموقع**](https://shawady6.github.io/baddel/) ·
[**Download / تحميل**](https://github.com/shawady6/baddel/raw/main/docs/downloads/baddel.zip) ·
[Privacy](PRIVACY.md)

</div>

---

## العربي

**بدّل** إضافة مجانية ومفتوحة المصدر لمتصفحات Chromium (Chrome وEdge وBrave وOpera) بتحوّل الكلام اللي اتكتب بلغة الكيبورد الغلط بين العربي والإنجليزي.

مثال: `hgsghl ugd;l` ← **السلام عليكم**

### المميزات
- اختصار واحد (الافتراضي `Ctrl+Shift+X`) بيحوّل النص المحدد أو آخر كلمة قبل المؤشر.
- بعد التحويل النص بيفضل متحدد، وضغطة تانية بترجّعه.
- الاختصار بيشتغل مهما كانت لغة الكيبورد الحالية.
- صفحة إعدادات: تسجيل اختصار جديد، اتجاه التحويل، تشغيل/إيقاف، وصندوق تجربة.
- خصوصية كاملة: مفيش أي طلبات شبكة ولا جمع بيانات.

### التثبيت (يدوي)
1. حمّل `baddel.zip` من [هنا](https://github.com/shawady6/baddel/raw/main/docs/downloads/baddel.zip) وفك ضغطه في فولدر ثابت.
2. افتح `chrome://extensions` (أو `edge://extensions`) وفعّل **Developer mode**.
3. دوس **Load unpacked** واختار فولدر `baddel`.
4. ثبّت الأيقونة من قايمة الإضافات 🧩.

## English

**Baddel** is a free, open-source extension for Chromium browsers (Chrome, Edge, Brave, Opera) that converts text typed in the wrong keyboard layout between Arabic and English.

### Features
- One shortcut (default `Ctrl+Shift+X`) converts the selection, or the word before the caret.
- The result stays selected, so pressing the shortcut again undoes it.
- The shortcut is matched by physical key, so it works whichever layout is active.
- Settings page: shortcut recorder, conversion direction, on/off switch, live preview.
- Private: no network requests, no data collection.

### Install (manual)
1. Download `baddel.zip` from the [here](https://github.com/shawady6/baddel/raw/main/docs/downloads/baddel.zip) and extract it somewhere permanent.
2. Open `chrome://extensions` (or `edge://extensions`) and turn on **Developer mode**.
3. Click **Load unpacked** and pick the `baddel` folder.

### Limitations
- Doesn't work on `chrome://` pages, the Chrome Web Store, canvas editors such as Google Docs, or email/password fields.
- The layout map is Arabic 101 ↔ US QWERTY. Rare shifted symbols may need a check after conversion.
- In the Arabic→English direction, the lam-alef ligature `لا` becomes `gh` (ل + ا), because the same characters come from two separate keys.

## Development

```text
extension/   the browser extension (load this folder unpacked)
docs/        the website (GitHub Pages, served from /docs)
scripts/     release.py  → builds dist/baddel.zip and syncs docs/shared.js
tests/       node tests for the conversion map and shortcut logic
```

```bash
node tests/test_logic.js        # unit tests
python3 scripts/release.py      # build dist/baddel.zip + docs/downloads/baddel.zip
```

`docs/shared.js` is a copy of `extension/shared.js` so the website demo uses the exact same conversion map. `release.py` keeps them in sync, and the test suite fails if they differ.

### Publishing a new version
```bash
# bump "version" in extension/manifest.json, then:
python3 scripts/release.py
node tests/test_logic.js
git add -A && git commit -m "Release vX.Y.Z" && git push
```
`docs/downloads/baddel.zip` is served by GitHub Pages, which is where the website's download button points.

## License
[MIT](LICENSE)

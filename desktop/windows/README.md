# Baddel for Windows · بدّل للويندوز

Convert text typed in the wrong keyboard language (Arabic ↔ English) **in any app** —
including the browser address bar, Word, WhatsApp, Notepad. It sits in the system tray and
listens for a global shortcut (default `Ctrl+Shift+X`).

## الاستخدام

1. ثبّت **AutoHotkey v2** (مجاني): https://www.autohotkey.com
2. دبل كليك على `baddel.ahk`. هتظهر أيقونة جنب الساعة.
3. حدد الكلام اللي اتكتب غلط واضغط `Ctrl+Shift+X`.
   لو مفيش تحديد، بيحوّل آخر كلمة قبل المؤشر.
4. ضغطة تانية بترجّع الكلام زي ما كان.

**عايز ملف `.exe` من غير ما تحتاج AutoHotkey على الأجهزة التانية؟**
كليك يمين على `baddel.ahk` ← **Compile script**، وبيطلع لك `baddel.exe` (جاي مع AutoHotkey).

دبل كليك على الأيقونة بتفتح نافذة **Settings**: تغيير الاختصار، اتجاه التحويل، تجربة مباشرة، والتشغيل مع الويندوز، وخيار **تبديل لغة الكيبورد تلقائيًا** (بعد التحويل لعربي يتحول الكيبورد لعربي، وللإنجليزي يتحول لإنجليزي).
الإعدادات بتتحفظ في `%AppData%\Baddel\settings.ini`.

## Usage

1. Install **AutoHotkey v2** (free): https://www.autohotkey.com
2. Double-click `baddel.ahk`; a tray icon appears.
3. Select the wrongly typed text and press `Ctrl+Shift+X`. With nothing selected, the last word before the caret is converted.
4. Press it again to undo.

To get a standalone `baddel.exe`: right-click `baddel.ahk` → **Compile script**.

Double-click the tray icon to open **Settings**: shortcut, direction, a live try-it box, start with Windows, and an option to **switch the keyboard language** to match the result.
Settings are stored in `%AppData%\Baddel\settings.ini`.

## لو الاختصار وقف

كليك يمين على الأيقونة ← **Settings...** ← **Default** ← **Save**.
ولو الأيقونة نفسها مش ظاهرة: امسح `%AppData%\Baddel\settings.ini` وشغّل البرنامج تاني.

If the shortcut stops working: tray icon → **Settings...** → **Default** → **Save**.
If the app won't start properly, delete `%AppData%\Baddel\settings.ini` and run it again.

## Notes

- The shortcut is bound to the physical key, so it works whichever keyboard language is active.

- If the target app runs as **Administrator**, run Baddel as Administrator too.
- It works by copying the selection, converting it and pasting it back, then restoring your
  clipboard. Terminals and some protected apps/games may block this.
- In right-to-left text fields, select the text first: arrow keys move visually there, so
  "last word" detection can pick the wrong side.
- If you also use the browser extension, you don't need both. If you do keep both, give them
  different shortcuts.
- The conversion tables are generated from `extension/shared.js` (`node scripts/gen_ahk.js`),
  so the app and the extension always convert identically.

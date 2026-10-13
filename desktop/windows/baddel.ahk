#Requires AutoHotkey v2.0
#SingleInstance Force
Persistent

;@Ahk2Exe-SetMainIcon baddel.ico
;@Ahk2Exe-SetName Baddel
;@Ahk2Exe-SetDescription Baddel - Arabic/English keyboard fixer
;@Ahk2Exe-SetVersion 1.1.0

; Baddel (بدّل) for Windows
; Select text typed in the wrong keyboard language, press the shortcut, done.
; Works in any app, including the browser address bar.
; https://github.com/shawady6/baddel

APP := "Baddel"
VERSION := "1.1.0"
SITE := "https://shawady6.github.io/baddel/"
DEFAULT_HK := "^+x"
SETTINGS_DIR := A_AppData "\Baddel"
INI := SETTINGS_DIR "\settings.ini"
STARTUP_LNK := A_Startup "\Baddel.lnk"

global gEnabled := true
global gLastWord := true
global gDirection := "auto"
global gHotkey := ""      ; as shown to the user, e.g. ^\
global gHotkeyReg := ""   ; what is registered: physical key (scan code), e.g. ^sc02B
global gDialog := ""

; ---------------------------------------------------------------------------
; Conversion tables. GENERATED from extension/shared.js by scripts/gen_ahk.js.
; Do not edit by hand: run  node scripts/gen_ahk.js
; ---------------------------------------------------------------------------
enToAr := Map()
enToAr.CaseSense := true
arToEn := Map()
arToEn.CaseSense := true
;@@MAPS_BEGIN
enToAr["'"] := "ط"
enToAr[","] := "و"
enToAr["."] := "ز"
enToAr["/"] := "ظ"
enToAr[";"] := "ك"
enToAr["?"] := "؟"
enToAr["A"] := "ِ"
enToAr["B"] := "لآ"
enToAr["C"] := "{"
enToAr["D"] := "]"
enToAr["E"] := "ُ"
enToAr["F"] := "["
enToAr["G"] := "لأ"
enToAr["H"] := "أ"
enToAr["I"] := "÷"
enToAr["J"] := "ـ"
enToAr["K"] := "،"
enToAr["L"] := "/"
enToAr["M"] := "’"
enToAr["N"] := "آ"
enToAr["O"] := "×"
enToAr["P"] := "؛"
enToAr["Q"] := "َ"
enToAr["R"] := "ٌ"
enToAr["S"] := "ٍ"
enToAr["T"] := "لإ"
enToAr["U"] := "‘"
enToAr["V"] := "}"
enToAr["W"] := "ً"
enToAr["X"] := "ْ"
enToAr["Y"] := "إ"
enToAr["Z"] := "~"
enToAr["["] := "ج"
enToAr["]"] := "د"
enToAr["``"] := "ذ"
enToAr["a"] := "ش"
enToAr["b"] := "لا"
enToAr["c"] := "ؤ"
enToAr["d"] := "ي"
enToAr["e"] := "ث"
enToAr["f"] := "ب"
enToAr["g"] := "ل"
enToAr["h"] := "ا"
enToAr["i"] := "ه"
enToAr["j"] := "ت"
enToAr["k"] := "ن"
enToAr["l"] := "م"
enToAr["m"] := "ة"
enToAr["n"] := "ى"
enToAr["o"] := "خ"
enToAr["p"] := "ح"
enToAr["q"] := "ض"
enToAr["r"] := "ق"
enToAr["s"] := "س"
enToAr["t"] := "ف"
enToAr["u"] := "ع"
enToAr["v"] := "ر"
enToAr["w"] := "ص"
enToAr["x"] := "ء"
enToAr["y"] := "غ"
enToAr["z"] := "ئ"
enToAr["~"] := "ّ"
arToEn["ط"] := "'"
arToEn["و"] := ","
arToEn["ز"] := "."
arToEn["ظ"] := "/"
arToEn["ك"] := ";"
arToEn["؟"] := "?"
arToEn["ِ"] := "A"
arToEn["ُ"] := "E"
arToEn["أ"] := "H"
arToEn["ـ"] := "J"
arToEn["،"] := "K"
arToEn["آ"] := "N"
arToEn["؛"] := "P"
arToEn["َ"] := "Q"
arToEn["ٌ"] := "R"
arToEn["ٍ"] := "S"
arToEn["ً"] := "W"
arToEn["ْ"] := "X"
arToEn["إ"] := "Y"
arToEn["ج"] := "["
arToEn["د"] := "]"
arToEn["ذ"] := "``"
arToEn["ش"] := "a"
arToEn["ؤ"] := "c"
arToEn["ي"] := "d"
arToEn["ث"] := "e"
arToEn["ب"] := "f"
arToEn["ل"] := "g"
arToEn["ا"] := "h"
arToEn["ه"] := "i"
arToEn["ت"] := "j"
arToEn["ن"] := "k"
arToEn["م"] := "l"
arToEn["ة"] := "m"
arToEn["ى"] := "n"
arToEn["خ"] := "o"
arToEn["ح"] := "p"
arToEn["ض"] := "q"
arToEn["ق"] := "r"
arToEn["س"] := "s"
arToEn["ف"] := "t"
arToEn["ع"] := "u"
arToEn["ر"] := "v"
arToEn["ص"] := "w"
arToEn["ء"] := "x"
arToEn["غ"] := "y"
arToEn["ئ"] := "z"
arToEn["ّ"] := "~"
;@@MAPS_END

; ---------------------------------------------------------------------------
; Start-up
; ---------------------------------------------------------------------------
LoadSettings()
if FileExist(A_ScriptDir "\baddel.ico")
    TraySetIcon(A_ScriptDir "\baddel.ico")
if !SetHotkey(gHotkey, gHotkeyReg) {
    ; the saved shortcut is unusable: fall back to the default
    gHotkey := ""
    gHotkeyReg := ""
    SetHotkey(DEFAULT_HK, PhysicalForm(DEFAULT_HK))
}
BuildMenu()
TrayTip("Select the wrong text and press " Friendly(gHotkey), APP " is running")
return

; ---------------------------------------------------------------------------
; Settings
; ---------------------------------------------------------------------------
LoadSettings() {
    global gEnabled, gLastWord, gDirection, gHotkey, gHotkeyReg
    gEnabled := IniRead(INI, "main", "enabled", "1") = "1"
    gLastWord := IniRead(INI, "main", "lastword", "1") = "1"
    gDirection := IniRead(INI, "main", "direction", "auto")
    if !(gDirection = "auto" || gDirection = "ar2en" || gDirection = "en2ar")
        gDirection := "auto"
    gHotkey := IniRead(INI, "main", "hotkey", DEFAULT_HK)
    if (gHotkey = "")
        gHotkey := DEFAULT_HK
    gHotkeyReg := IniRead(INI, "main", "hotkeyreg", "")
    if (gHotkeyReg = "")
        gHotkeyReg := PhysicalForm(gHotkey)
}

SaveSettings() {
    try {
        DirCreate(SETTINGS_DIR)
        IniWrite(gEnabled ? "1" : "0", INI, "main", "enabled")
        IniWrite(gLastWord ? "1" : "0", INI, "main", "lastword")
        IniWrite(gDirection, INI, "main", "direction")
        IniWrite(gHotkey, INI, "main", "hotkey")
        IniWrite(gHotkeyReg, INI, "main", "hotkeyreg")
    }
}

; "^\" -> "^sc02B": bind to the physical key instead of the character, so the
; shortcut keeps working whichever keyboard language is active.
PhysicalForm(hk) {
    mods := RegExReplace(hk, "^([+^!#]*).*$", "$1")
    key := SubStr(hk, StrLen(mods) + 1)
    if (key = "")
        return hk
    try {
        sc := GetKeySC(key)
    } catch {
        return hk
    }
    if (sc = 0)
        return hk
    return mods "sc" Format("{:03X}", sc)
}

; Registers the global hotkey (reg = physical form). "$" makes it use the keyboard
; hook so the keys we send ourselves can never re-trigger it.
SetHotkey(hk, reg) {
    global gHotkey, gHotkeyReg
    oldReg := gHotkeyReg
    if (hk = "" || reg = "")
        return false
    try {
        Hotkey("$" reg, Fix, "On")
    } catch {
        return false
    }
    if (oldReg != "" && oldReg != reg) {
        try Hotkey("$" oldReg, "Off")
    }
    gHotkey := hk
    gHotkeyReg := reg
    return true
}

; "^+x" -> "Ctrl+Shift+X"
Friendly(hk) {
    out := ""
    k := hk
    while (k != "") {
        c := SubStr(k, 1, 1)
        if (c = "^")
            out .= "Ctrl+"
        else if (c = "!")
            out .= "Alt+"
        else if (c = "+")
            out .= "Shift+"
        else if (c = "#")
            out .= "Win+"
        else
            break
        k := SubStr(k, 2)
    }
    last := (StrLen(k) = 1) ? StrUpper(k) : k
    return out last
}

; ---------------------------------------------------------------------------
; Tray menu
; ---------------------------------------------------------------------------
BuildMenu() {
    global gEnabled, gDirection, gHotkey
    m := A_TrayMenu
    m.Delete()

    head := APP " " VERSION "  (" Friendly(gHotkey) ")"
    m.Add(head, (*) => 0)
    m.Disable(head)
    m.Add()

    m.Add("Enabled", (*) => Toggle("enabled"))
    if gEnabled
        m.Check("Enabled")

    dm := Menu()
    dm.Add("Auto-detect", (*) => SetDirection("auto"))
    dm.Add("Arabic to English", (*) => SetDirection("ar2en"))
    dm.Add("English to Arabic", (*) => SetDirection("en2ar"))
    if (gDirection = "auto")
        dm.Check("Auto-detect")
    else if (gDirection = "ar2en")
        dm.Check("Arabic to English")
    else
        dm.Check("English to Arabic")
    m.Add("Direction", dm)

    m.Add("Settings...", (*) => OpenSettings())
    m.Add()
    m.Add("Open website", (*) => Run(SITE))
    m.Add("Exit", (*) => ExitApp())

    m.Default := "Settings..."
    m.ClickCount := 1
    A_IconTip := APP " - " Friendly(gHotkey)
}

Toggle(what) {
    global gEnabled
    gEnabled := !gEnabled
    SaveSettings()
    BuildMenu()
}

SetDirection(d) {
    global gDirection
    gDirection := d
    SaveSettings()
    BuildMenu()
}

SetStartup(on) {
    try {
        exists := FileExist(STARTUP_LNK)
        if (on && !exists)
            FileCreateShortcut(A_ScriptFullPath, STARTUP_LNK, A_ScriptDir, , APP)
        else if (!on && exists)
            FileDelete(STARTUP_LNK)
    }
}

; Returns an error message for an unusable shortcut, or "" when it is fine.
HotkeyError(hk) {
    if (hk = "")
        return "Press a shortcut first."
    mods := RegExReplace(hk, "^([+^!#]*).*$", "$1")
    if !(InStr(mods, "^") || InStr(mods, "!") || InStr(mods, "#"))
        return "Add Ctrl, Alt or Win to the shortcut."
    low := StrLower(hk)
    for r in ["^c", "^v", "^x", "^z", "^y", "^a", "^s", "^t", "^w", "^n", "^+t", "^+n", "^+w", "!f4"] {
        if (low = r)
            return "This shortcut is reserved by Windows or the browser."
    }
    return ""
}

; ---------------------------------------------------------------------------
; Settings window
; ---------------------------------------------------------------------------
OpenSettings(*) {
    global gHotkey, gHotkeyReg, gEnabled, gLastWord, gDirection, gDialog
    if IsObject(gDialog) {
        gDialog.Show()
        return
    }

    ; nested functions come first; they use the controls created below
    Preview(*) {
        mode := r1.Value ? "auto" : (r2.Value ? "ar2en" : "en2ar")
        outBox.Value := Convert(inBox.Value, mode)
    }

    CloseDlg(*) {
        global gHotkeyReg, gDialog
        try Hotkey("$" gHotkeyReg, "On")
        gDialog := ""
        g.Destroy()
    }

    SaveDlg(*) {
        global gHotkey, gHotkeyReg, gEnabled, gLastWord, gDirection, gDialog
        hk := hkc.Value
        err := HotkeyError(hk)
        if (err != "") {
            msg.Value := err
            return
        }
        try Hotkey("$" gHotkeyReg, "On")
        if (hk != gHotkey) {
            if !SetHotkey(hk, PhysicalForm(hk)) {
                try Hotkey("$" gHotkeyReg, "Off")
                msg.Value := "Windows does not accept this shortcut."
                return
            }
        }
        gEnabled := cbOn.Value = 1
        gLastWord := cbLast.Value = 1
        gDirection := r1.Value ? "auto" : (r2.Value ? "ar2en" : "en2ar")
        SetStartup(cbStart.Value = 1)
        SaveSettings()
        BuildMenu()
        gDialog := ""
        g.Destroy()
        TrayTip("Shortcut: " Friendly(gHotkey) ". Select some text and try it.", APP " - saved")
    }

    ; the global hotkey must not fire while the shortcut is being recorded
    try Hotkey("$" gHotkeyReg, "Off")

    g := Gui("+AlwaysOnTop -MinimizeBox -MaximizeBox", APP " - Settings")
    gDialog := g
    g.BackColor := "F4F5FB"
    g.MarginX := 16
    g.MarginY := 10
    g.SetFont("s10", "Segoe UI")

    ; header band with the app icon
    g.Add("Text", "x0 y0 w460 h76 Background4F46E5", "")
    if FileExist(A_ScriptDir "\baddel.ico")
        g.Add("Picture", "x18 y16 w44 h44 BackgroundTrans", A_ScriptDir "\baddel.ico")
    g.SetFont("s16 bold cFFFFFF", "Segoe UI")
    g.Add("Text", "x76 y12 w360 BackgroundTrans", "Baddel  ·  بدّل")
    g.SetFont("s9 norm cFFFFFF", "Segoe UI")
    g.Add("Text", "x76 y44 w360 BackgroundTrans", "Fix text typed in the wrong keyboard language")
    g.SetFont("s10 norm c000000", "Segoe UI")

    ; shortcut
    g.SetFont("s10 bold", "Segoe UI")
    g.Add("Text", "xm y+20 w428", "Shortcut")
    g.SetFont("s9 norm c5D6980", "Segoe UI")
    g.Add("Text", "xm y+2 w428", "Click the box, then press the new keys (Ctrl, Alt or Win + a key).")
    g.SetFont("s10 norm c000000", "Segoe UI")
    hkc := g.Add("Hotkey", "xm y+8 w300 h28", gHotkey)
    bDef := g.Add("Button", "x+10 yp w118 h28", "Default")
    bDef.OnEvent("Click", (*) => hkc.Value := DEFAULT_HK)
    g.SetFont("s9 norm cC0392B", "Segoe UI")
    msg := g.Add("Text", "xm y+4 w428 h20", "")
    g.SetFont("s10 norm c000000", "Segoe UI")

    ; direction
    g.SetFont("s10 bold", "Segoe UI")
    g.Add("Text", "xm y+8 w428", "Direction")
    g.SetFont("s10 norm c000000", "Segoe UI")
    r1 := g.Add("Radio", "xm y+6 Group", "Auto-detect")
    r2 := g.Add("Radio", "x+16 yp", "Arabic → English")
    r3 := g.Add("Radio", "x+16 yp", "English → Arabic")
    r1.Value := (gDirection = "auto") ? 1 : 0
    r2.Value := (gDirection = "ar2en") ? 1 : 0
    r3.Value := (gDirection = "en2ar") ? 1 : 0
    r1.OnEvent("Click", Preview)
    r2.OnEvent("Click", Preview)
    r3.OnEvent("Click", Preview)

    ; options
    g.SetFont("s10 bold", "Segoe UI")
    g.Add("Text", "xm y+16 w428", "Options")
    g.SetFont("s10 norm c000000", "Segoe UI")
    cbOn := g.Add("Checkbox", "xm y+6 w428", "Enabled")
    cbOn.Value := gEnabled ? 1 : 0
    cbLast := g.Add("Checkbox", "xm y+4 w428", "Convert the last word when nothing is selected")
    cbLast.Value := gLastWord ? 1 : 0
    cbStart := g.Add("Checkbox", "xm y+4 w428", "Start with Windows")
    cbStart.Value := FileExist(STARTUP_LNK) ? 1 : 0

    ; live preview
    g.SetFont("s10 bold", "Segoe UI")
    g.Add("Text", "xm y+16 w428", "Try it")
    g.SetFont("s10 norm c000000", "Segoe UI")
    inBox := g.Add("Edit", "xm y+6 w208 r3", "hgsghl ugd;l")
    outBox := g.Add("Edit", "x+12 yp w208 r3 ReadOnly", "")
    inBox.OnEvent("Change", Preview)
    Preview()

    ; buttons
    bSave := g.Add("Button", "xm+228 y+20 w100 h32 Default", "Save")
    bCancel := g.Add("Button", "x+8 yp w100 h32", "Cancel")
    bSave.OnEvent("Click", SaveDlg)
    bCancel.OnEvent("Click", CloseDlg)
    g.OnEvent("Close", CloseDlg)
    g.OnEvent("Escape", CloseDlg)

    g.SetFont("s8 norm c8A94A8", "Segoe UI")
    g.Add("Text", "xm y+14 w428", APP " " VERSION "  ·  shawady6.github.io/baddel")
    g.Show("w460")
}

; ---------------------------------------------------------------------------
; Conversion
; ---------------------------------------------------------------------------
Convert(text, mode := "") {
    if (mode = "")
        mode := gDirection
    if (mode = "auto") {
        ar := StrLen(RegExReplace(text, "[^\x{0600}-\x{06FF}]"))
        la := StrLen(RegExReplace(text, "[^A-Za-z]"))
        if (ar = 0 && la = 0)
            return text
        mode := (ar >= la) ? "ar2en" : "en2ar"
    }
    tbl := (mode = "ar2en") ? arToEn : enToAr
    out := ""
    for ch in StrSplit(text)
        out .= tbl.Has(ch) ? tbl[ch] : ch
    return out
}

; ---------------------------------------------------------------------------
; The hotkey action
; ---------------------------------------------------------------------------
Fix(*) {
    if !gEnabled
        return
    ; wait until the shortcut keys are released, otherwise our Ctrl+C would
    ; arrive as Ctrl+Shift+C and so on
    for k in ["Ctrl", "Shift", "Alt", "LWin", "RWin"]
        KeyWait(k, "T1")
    mainKey := RegExReplace(A_ThisHotkey, "^[$~*<>^!+#]+")
    if (mainKey != "") {
        try KeyWait(mainKey, "T1")
    }
    saved := ClipboardAll()
    try {
        DoFix()
    }
    Sleep(200)
    A_Clipboard := saved
}

CopySelection() {
    A_Clipboard := ""
    Send("^c")
    if !ClipWait(0.3)
        return ""
    return A_Clipboard
}

DoFix() {
    madeSelection := false
    sel := CopySelection()
    if (sel = "") {
        if !gLastWord
            return
        sel := SelectLastToken()
        if (sel = "")
            return
        madeSelection := true
    }

    RegExMatch(sel, "s)^(.*?)(\s*)$", &mm)
    core := mm[1]
    trail := mm[2]
    if (core = "") {
        if madeSelection
            Send("{Right}")
        return
    }

    out := Convert(core) trail
    if (out == sel) {
        if madeSelection
            Send("{Right}")
        return
    }

    A_Clipboard := out
    if !ClipWait(0.5)
        return
    Send("^v")
    Sleep(150)

    ; keep the result selected so pressing the shortcut again flips it back
    n := StrLen(StrReplace(out, "`r`n", "`n"))
    if (n > 0 && n < 2000)
        Send("+{Left " n "}")
}

; With nothing selected: select the last whitespace-delimited word before the
; caret (plus any trailing spaces). Ctrl+Shift+Left stops at punctuation in many
; apps, and several Arabic letters are typed with punctuation keys, so keep
; extending until a whitespace boundary is crossed, then step back.
SelectLastToken() {
    prev := ""
    Loop 8 {
        Send("^+{Left}")
        Sleep(30)
        cur := CopySelection()
        if (cur = "")
            break
        if (cur == prev)
            break
        trimmed := RTrim(cur, " `t`r`n")
        pos := 0
        p := 0
        while (pos := RegExMatch(trimmed, "\s", , pos + 1))
            p := pos
        if (p > 0) {
            steps := StrLen(StrReplace(SubStr(cur, 1, p), "`r`n", "`n"))
            Send("+{Right " steps "}")
            Sleep(30)
            return CopySelection()
        }
        prev := cur
    }
    return prev
}

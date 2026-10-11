#Requires AutoHotkey v2.0
#SingleInstance Force
Persistent

;@Ahk2Exe-SetMainIcon baddel.ico
;@Ahk2Exe-SetName Baddel
;@Ahk2Exe-SetDescription Baddel - Arabic/English keyboard fixer
;@Ahk2Exe-SetVersion 1.0.0

; Baddel (بدّل) for Windows
; Select text typed in the wrong keyboard language, press the shortcut, done.
; Works in any app, including the browser address bar.
; https://github.com/shawady6/baddel

APP := "Baddel"
VERSION := "1.0.0"
SITE := "https://shawady6.github.io/baddel/"
DEFAULT_HK := "^+x"
SETTINGS_DIR := A_AppData "\Baddel"
INI := SETTINGS_DIR "\settings.ini"
STARTUP_LNK := A_Startup "\Baddel.lnk"

global gEnabled := true
global gLastWord := true
global gDirection := "auto"
global gHotkey := ""
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
if !SetHotkey(gHotkey) {
    ; the saved shortcut is unusable: fall back to the default
    gHotkey := ""
    SetHotkey(DEFAULT_HK)
}
BuildMenu()
TrayTip("Select the wrong text and press " Friendly(gHotkey), APP " is running")
return

; ---------------------------------------------------------------------------
; Settings
; ---------------------------------------------------------------------------
LoadSettings() {
    global gEnabled, gLastWord, gDirection, gHotkey
    gEnabled := IniRead(INI, "main", "enabled", "1") = "1"
    gLastWord := IniRead(INI, "main", "lastword", "1") = "1"
    gDirection := IniRead(INI, "main", "direction", "auto")
    if !(gDirection = "auto" || gDirection = "ar2en" || gDirection = "en2ar")
        gDirection := "auto"
    gHotkey := IniRead(INI, "main", "hotkey", DEFAULT_HK)
    if (gHotkey = "")
        gHotkey := DEFAULT_HK
}

SaveSettings() {
    try {
        DirCreate(SETTINGS_DIR)
        IniWrite(gEnabled ? "1" : "0", INI, "main", "enabled")
        IniWrite(gLastWord ? "1" : "0", INI, "main", "lastword")
        IniWrite(gDirection, INI, "main", "direction")
        IniWrite(gHotkey, INI, "main", "hotkey")
    }
}

; Registers the global hotkey. "$" makes it use the keyboard hook so the keys we
; send ourselves can never re-trigger it.
SetHotkey(hk) {
    global gHotkey
    old := gHotkey
    if (hk = "")
        return false
    try {
        Hotkey("$" hk, Fix, "On")
    } catch {
        return false
    }
    if (old != "" && old != hk) {
        try Hotkey("$" old, "Off")
    }
    gHotkey := hk
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
    global gEnabled, gLastWord, gDirection, gHotkey
    m := A_TrayMenu
    m.Delete()

    head := APP " " VERSION "  (" Friendly(gHotkey) ")"
    m.Add(head, (*) => 0)
    m.Disable(head)
    m.Add()

    m.Add("Enabled", (*) => Toggle("enabled"))
    if gEnabled
        m.Check("Enabled")

    m.Add("Convert last word if nothing is selected", (*) => Toggle("lastword"))
    if gLastWord
        m.Check("Convert last word if nothing is selected")

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

    m.Add("Change shortcut...", (*) => ChangeShortcut())
    m.Add()
    m.Add("Start with Windows", (*) => ToggleStartup())
    if FileExist(STARTUP_LNK)
        m.Check("Start with Windows")
    m.Add("Open website", (*) => Run(SITE))
    m.Add("Exit", (*) => ExitApp())

    A_IconTip := APP " - " Friendly(gHotkey)
}

Toggle(what) {
    global gEnabled, gLastWord
    if (what = "enabled")
        gEnabled := !gEnabled
    else
        gLastWord := !gLastWord
    SaveSettings()
    BuildMenu()
}

SetDirection(d) {
    global gDirection
    gDirection := d
    SaveSettings()
    BuildMenu()
}

ToggleStartup() {
    try {
        if FileExist(STARTUP_LNK)
            FileDelete(STARTUP_LNK)
        else
            FileCreateShortcut(A_ScriptFullPath, STARTUP_LNK, A_ScriptDir, , APP)
    }
    BuildMenu()
}

ChangeShortcut() {
    global gHotkey, gDialog
    if IsObject(gDialog) {
        gDialog.Show()
        return
    }

    ; nested functions are defined first; they use the variables declared below
    CloseDlg(*) {
        global gHotkey, gDialog
        try Hotkey("$" gHotkey, "On")
        gDialog := ""
        g.Destroy()
    }

    SaveDlg(*) {
        global gHotkey, gDialog
        hk := hkc.Value
        if (hk = "") {
            msg.Value := "Press a shortcut first."
            return
        }
        mods := RegExReplace(hk, "^([+^!#]*).*$", "$1")
        if !(InStr(mods, "^") || InStr(mods, "!") || InStr(mods, "#")) {
            msg.Value := "Add Ctrl, Alt or Win to the shortcut."
            return
        }
        reserved := ["^c", "^v", "^x", "^z", "^y", "^a", "^s", "^t", "^w", "^n", "^+t", "^+n", "^+w", "!f4"]
        low := StrLower(hk)
        for r in reserved {
            if (low = r) {
                msg.Value := "This shortcut is reserved by Windows or the browser."
                return
            }
        }
        try Hotkey("$" gHotkey, "On")
        if !SetHotkey(hk) {
            try Hotkey("$" gHotkey, "Off")
            msg.Value := "Windows does not accept this shortcut."
            return
        }
        SaveSettings()
        BuildMenu()
        gDialog := ""
        g.Destroy()
    }

    ; while the dialog is open the global hotkey must not fire
    try Hotkey("$" gHotkey, "Off")

    g := Gui("+AlwaysOnTop -MinimizeBox", APP " - Change shortcut")
    gDialog := g
    g.SetFont("s10", "Segoe UI")
    g.Add("Text", "w320", "Click the box, then press the new shortcut.`nIt must include Ctrl, Alt or Win.")
    hkc := g.Add("Hotkey", "w320", gHotkey)
    msg := g.Add("Text", "w320 h24 cRed", "")
    bSave := g.Add("Button", "Default w100", "Save")
    bDef := g.Add("Button", "x+8 w100", "Default")
    bCancel := g.Add("Button", "x+8 w100", "Cancel")
    bSave.OnEvent("Click", SaveDlg)
    bDef.OnEvent("Click", (*) => hkc.Value := DEFAULT_HK)
    bCancel.OnEvent("Click", CloseDlg)
    g.OnEvent("Close", CloseDlg)
    g.OnEvent("Escape", CloseDlg)
    g.Show()
}

; ---------------------------------------------------------------------------
; Conversion
; ---------------------------------------------------------------------------
Convert(text) {
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

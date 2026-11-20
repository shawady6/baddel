//! Windows glue: tray icon, global shortcut, clipboard, keyboard input,
//! keyboard-layout switching and the "Start with Windows" registry entry.

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{GetLastError, HANDLE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardSequenceNumber,
    OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, GetAsyncKeyState, GetKeyboardLayoutList, MapVirtualKeyExW, HKL, RegisterHotKey, SendInput, UnregisterHotKey, INPUT, INPUT_0, INPUT_KEYBOARD,
    KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MOD_NOREPEAT, VK_CONTROL, VK_HOME, VK_LEFT, VK_LWIN,
    VK_MENU, VK_RIGHT, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::Shell::{
    DefSubclassProc, SetWindowSubclass, ShellExecuteW, Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_INFO, NIM_ADD, NIM_DELETE,
    NIM_MODIFY, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, GetForegroundWindow, GetMessageW, LoadIconW, PostMessageW, PostQuitMessage,
    RegisterClassW, RegisterWindowMessageW, SetForegroundWindow, TrackPopupMenu, TranslateMessage,
    IDI_APPLICATION, MF_CHECKED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG, TPM_BOTTOMALIGN, TPM_RIGHTBUTTON,
    WM_APP, WM_COMMAND, WM_CONTEXTMENU, WM_DESTROY, WM_HOTKEY, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP,
    WNDCLASSW, AdjustWindowRectEx, DestroyWindow, GetDlgItem,
    GetDlgItemTextW, GetSystemMetrics, IsDialogMessageW, MessageBoxW, SendMessageW,
    SetDlgItemTextW, ShowWindow, BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON, BS_DEFPUSHBUTTON, BS_GROUPBOX,
    BS_PUSHBUTTON, ES_AUTOHSCROLL, ES_READONLY, IDCANCEL, IDOK, MB_ICONWARNING, MB_OK, SM_CXSCREEN, SM_CYSCREEN,
    SW_SHOW, KillTimer, SetTimer, WM_TIMER, WM_CHAR, WM_KEYDOWN, WM_SYSCHAR, WM_SYSKEYDOWN, SetWindowTextW, WM_CLOSE, WM_LBUTTONDBLCLK, WM_SETFONT, WS_CAPTION, WS_CHILD, WS_EX_CLIENTEDGE, WS_GROUP,
    WS_MINIMIZEBOX, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};
use windows_sys::Win32::Graphics::Gdi::{CreateFontW, DeleteObject, GetSysColorBrush, COLOR_BTNFACE};
use windows_sys::Win32::UI::Controls::{CheckDlgButton, CheckRadioButton, IsDlgButtonChecked};
use windows_sys::Win32::UI::HiDpi::GetDpiForSystem;
use windows_sys::Win32::Foundation::RECT;

use crate::core::{
    convert, dominant_script, format_hotkey, hotkey_text, key_steps, last_unit, parse_hotkey, Direction, LastMode,
    Script, Settings, DEFAULT_HOTKEY,
};

const APP_NAME: &str = "Baddel";
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const WEBSITE: &str = "https://shawady6.github.io/baddel/";
const WM_TRAY: u32 = WM_APP + 1;
const WM_INPUTLANGCHANGEREQUEST: u32 = 0x0050;
const HOTKEY_ID: i32 = 1;
const HOTKEY_SLOTS: i32 = 16;
const TIMER_LAYOUTS: usize = 77;
const CF_UNICODETEXT: u32 = 13;

// menu command ids
const ID_SETTINGS: usize = 1000;
const ID_ENABLED: usize = 1001;
const ID_LASTWORD: usize = 1002;
const ID_SWITCHLANG: usize = 1003;
const ID_STARTUP: usize = 1004;
const ID_DIR_AUTO: usize = 1010;
const ID_DIR_AR2EN: usize = 1011;
const ID_DIR_EN2AR: usize = 1012;
const ID_PRESET_BASE: usize = 1100;
const ID_OPEN_FILE: usize = 1030;
const ID_RELOAD: usize = 1031;
const ID_WEBSITE: usize = 1032;
const ID_EXIT: usize = 1099;

const PRESETS: [&str; 6] = [
    "Ctrl+Shift+X",
    "Ctrl+Alt+X",
    "Ctrl+Shift+Space",
    "Ctrl+\\",
    "Ctrl+Alt+Shift+K",
    "Ctrl+Alt+Space",
];

struct Undo {
    out: String,
    orig: String,
    at: Instant,
}

struct App {
    settings: Settings,
    undo: Option<Undo>,
}

static APP: Mutex<Option<App>> = Mutex::new(None);
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
static HWND_MAIN: Mutex<usize> = Mutex::new(0);

// ---------------------------------------------------------------------------
// small helpers
// ---------------------------------------------------------------------------

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn hwnd_main() -> HWND {
    *HWND_MAIN.lock().unwrap() as HWND
}

fn settings() -> Settings {
    APP.lock().unwrap().as_ref().map(|a| a.settings.clone()).unwrap_or_default()
}

fn settings_path() -> String {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    format!("{}\\{}\\settings.txt", base, APP_NAME)
}

fn load_settings() -> Settings {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(text) => Settings::parse(&text),
        Err(_) => {
            let s = Settings::default();
            save_settings(&s);
            s
        }
    }
}

fn save_settings(s: &Settings) {
    let path = settings_path();
    if let Some(dir) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&path, s.serialize());
}

fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

// ---------------------------------------------------------------------------
// Start with Windows (HKCU\...\Run, no admin needed)
// ---------------------------------------------------------------------------

fn exe_command() -> String {
    let exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    format!("\"{}\"", exe)
}

fn open_run_key(access: u32) -> Option<HKEY> {
    let mut key: HKEY = null_mut();
    let sub = wide(RUN_KEY);
    let rc = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, access, &mut key) };
    if rc == 0 {
        Some(key)
    } else {
        None
    }
}

/// The command currently stored in the Run key, if any.
fn startup_entry() -> Option<String> {
    let key = open_run_key(KEY_QUERY_VALUE)?;
    let name = wide(APP_NAME);
    let mut size: u32 = 0;
    let mut kind: u32 = 0;
    let rc = unsafe { RegQueryValueExW(key, name.as_ptr(), null(), &mut kind, null_mut(), &mut size) };
    let mut result = None;
    if rc == 0 && size >= 2 {
        let mut buf = vec![0u16; (size as usize) / 2 + 1];
        let mut size2 = size;
        let rc2 = unsafe {
            RegQueryValueExW(key, name.as_ptr(), null(), &mut kind, buf.as_mut_ptr() as *mut u8, &mut size2)
        };
        if rc2 == 0 {
            let n = (size2 as usize) / 2;
            let text = String::from_utf16_lossy(&buf[..n.min(buf.len())]);
            result = Some(text.trim_end_matches('\0').to_string());
        }
    }
    unsafe { RegCloseKey(key) };
    result
}

fn set_startup(on: bool) -> bool {
    let Some(key) = open_run_key(KEY_SET_VALUE) else { return false };
    let name = wide(APP_NAME);
    let ok = if on {
        let value = wide(&exe_command());
        let rc = unsafe {
            RegSetValueExW(key, name.as_ptr(), 0, REG_SZ, value.as_ptr() as *const u8, (value.len() * 2) as u32)
        };
        rc == 0
    } else {
        let rc = unsafe { RegDeleteValueW(key, name.as_ptr()) };
        rc == 0 || rc == 2 // 2 = it was not there
    };
    unsafe { RegCloseKey(key) };
    ok
}

// ---------------------------------------------------------------------------
// Keyboard input
// ---------------------------------------------------------------------------

fn is_extended(vk: u16) -> bool {
    (0x21..=0x28).contains(&vk) || vk == 0x2D || vk == 0x2E
}

fn key_event(vk: u16, up: bool) -> INPUT {
    let mut flags = if up { KEYEVENTF_KEYUP } else { 0 };
    if is_extended(vk) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 },
        },
    }
}

fn send(events: &[INPUT]) {
    if events.is_empty() {
        return;
    }
    unsafe {
        SendInput(events.len() as u32, events.as_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

/// Press `keys` (modifiers first) together, then release them in reverse order.
fn chord(keys: &[u16]) {
    let mut ev: Vec<INPUT> = keys.iter().map(|k| key_event(*k, false)).collect();
    ev.extend(keys.iter().rev().map(|k| key_event(*k, true)));
    send(&ev);
}

/// Hold Shift and tap `vk` `n` times.
fn shift_tap(vk: u16, n: usize) {
    if n == 0 {
        return;
    }
    let mut ev = vec![key_event(VK_SHIFT, false)];
    for _ in 0..n {
        ev.push(key_event(vk, false));
        ev.push(key_event(vk, true));
    }
    ev.push(key_event(VK_SHIFT, true));
    send(&ev);
}

fn is_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16) & 0x8000 != 0 }
}

/// Wait until the shortcut keys are physically released, otherwise our own Ctrl+C
/// would arrive as Ctrl+Shift+C and so on.
fn wait_release(main_vks: &[u32]) {
    let start = Instant::now();
    let mut watched = vec![VK_CONTROL as i32, VK_SHIFT as i32, VK_MENU as i32, VK_LWIN as i32, VK_RWIN as i32];
    watched.extend(main_vks.iter().map(|v| *v as i32));
    while start.elapsed() < Duration::from_millis(1500) {
        if !watched.iter().any(|vk| is_down(*vk)) {
            return;
        }
        sleep_ms(10);
    }
}

// ---------------------------------------------------------------------------
// Clipboard
// ---------------------------------------------------------------------------

fn open_clipboard() -> bool {
    for _ in 0..20 {
        if unsafe { OpenClipboard(null_mut()) } != 0 {
            return true;
        }
        sleep_ms(10);
    }
    false
}

fn read_clipboard_text() -> Option<String> {
    if !open_clipboard() {
        return None;
    }
    let mut result = None;
    unsafe {
        let h = GetClipboardData(CF_UNICODETEXT);
        if !h.is_null() {
            let p = GlobalLock(h) as *const u16;
            if !p.is_null() {
                let max = GlobalSize(h) / 2;
                let mut len = 0usize;
                while len < max && *p.add(len) != 0 {
                    len += 1;
                }
                result = Some(String::from_utf16_lossy(std::slice::from_raw_parts(p, len)));
                GlobalUnlock(h);
            }
        }
        CloseClipboard();
    }
    result
}

/// Put raw bytes on the clipboard under `format` (the clipboard must be open).
unsafe fn set_clipboard_bytes(format: u32, bytes: &[u8]) {
    let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1));
    if h.is_null() {
        return;
    }
    let p = GlobalLock(h) as *mut u8;
    if p.is_null() {
        return;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
    GlobalUnlock(h);
    if SetClipboardData(format, h as HANDLE).is_null() {
        // ownership stays with us when the call fails, but leaking a few bytes is harmless
    }
}

fn write_clipboard_text(text: &str) -> bool {
    if !open_clipboard() {
        return false;
    }
    unsafe {
        EmptyClipboard();
        let mut data: Vec<u8> = Vec::with_capacity(text.len() * 2 + 2);
        for u in text.encode_utf16().chain(std::iter::once(0)) {
            data.extend_from_slice(&u.to_le_bytes());
        }
        set_clipboard_bytes(CF_UNICODETEXT, &data);
        CloseClipboard();
    }
    true
}

struct SavedClipboard {
    items: Vec<(u32, Vec<u8>)>,
}

/// Remember every memory-backed clipboard format so the user's clipboard survives.
fn save_clipboard() -> SavedClipboard {
    let mut items = Vec::new();
    if open_clipboard() {
        unsafe {
            let mut fmt = EnumClipboardFormats(0);
            while fmt != 0 {
                let plain = matches!(fmt, 1 | 7 | 8 | 13 | 15 | 16 | 17) || fmt >= 0xC000;
                if plain {
                    let h = GetClipboardData(fmt);
                    if !h.is_null() {
                        let size = GlobalSize(h);
                        if size > 0 && size < 64 * 1024 * 1024 {
                            let p = GlobalLock(h) as *const u8;
                            if !p.is_null() {
                                items.push((fmt, std::slice::from_raw_parts(p, size).to_vec()));
                                GlobalUnlock(h);
                            }
                        }
                    }
                }
                fmt = EnumClipboardFormats(fmt);
            }
            CloseClipboard();
        }
    }
    SavedClipboard { items }
}

fn restore_clipboard(saved: SavedClipboard) {
    if !open_clipboard() {
        return;
    }
    unsafe {
        EmptyClipboard();
        for (fmt, bytes) in saved.items.iter() {
            set_clipboard_bytes(*fmt, bytes);
        }
        CloseClipboard();
    }
}

fn clear_clipboard() {
    if open_clipboard() {
        unsafe {
            EmptyClipboard();
            CloseClipboard();
        }
    }
}

/// Ctrl+C the current selection and return it (None when nothing was copied).
fn copy_selection() -> Option<String> {
    clear_clipboard();
    let seq0 = unsafe { GetClipboardSequenceNumber() };
    chord(&[VK_CONTROL, b'C' as u16]);
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(400) {
        if unsafe { GetClipboardSequenceNumber() } != seq0 {
            sleep_ms(15);
            return read_clipboard_text().filter(|t| !t.is_empty());
        }
        sleep_ms(10);
    }
    None
}

fn paste_text(text: &str) -> bool {
    if !write_clipboard_text(text) {
        return false;
    }
    chord(&[VK_CONTROL, b'V' as u16]);
    sleep_ms(150); // let the target read the clipboard before we restore it
    true
}

// ---------------------------------------------------------------------------
// Keyboard layout
// ---------------------------------------------------------------------------

fn switch_layout_for(text: &str) {
    let want: usize = match dominant_script(text) {
        Some(Script::Arabic) => 0x01,
        Some(Script::Latin) => 0x09,
        None => return,
    };
    unsafe {
        let n = GetKeyboardLayoutList(0, null_mut());
        if n < 1 {
            return;
        }
        let mut list: Vec<HKL> = vec![null_mut(); n as usize];
        let got = GetKeyboardLayoutList(n, list.as_mut_ptr());
        for hkl in list.iter().take(got.max(0) as usize) {
            if (*hkl as usize) & 0x3FF == want {
                PostMessageW(GetForegroundWindow(), WM_INPUTLANGCHANGEREQUEST, 0, *hkl as LPARAM);
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The shortcut action
// ---------------------------------------------------------------------------

fn norm(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn remember_undo(out: &str, orig: &str) {
    if let Some(app) = APP.lock().unwrap().as_mut() {
        app.undo = Some(Undo { out: out.to_string(), orig: orig.to_string(), at: Instant::now() });
    }
}

fn take_undo() -> Option<Undo> {
    let mut guard = APP.lock().unwrap();
    let u = guard.as_mut()?.undo.take()?;
    if u.at.elapsed() < Duration::from_secs(60) {
        Some(u)
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Ltr,
    Rtl,
}

fn guess_dir(text: &str) -> Dir {
    if dominant_script(text) == Some(Script::Arabic) {
        Dir::Rtl
    } else {
        Dir::Ltr
    }
}

/// Collapse a selection that ends at the original caret position.
fn collapse(dir: Dir) {
    chord(&[if dir == Dir::Ltr { VK_RIGHT } else { VK_LEFT }]);
}

/// Shift+Home selects from the logical start of the line to the caret (this works the same in
/// left-to-right and right-to-left text, unlike Ctrl+Shift+Left). Returns the selected text.
fn select_line_prefix() -> Option<String> {
    chord(&[VK_SHIFT, VK_HOME]);
    sleep_ms(25);
    copy_selection().map(|t| norm(&t))
}

/// With `prefix` selected (start of line .. caret), shrink the selection from its start by `steps`
/// characters until exactly `expected` is selected. The arrow key that moves the start inwards
/// depends on the text direction, so the first step is a probe.
fn shrink_to(prefix: &str, steps: usize, expected: &str) -> Result<(), Dir> {
    if steps == 0 {
        return Ok(());
    }
    shift_tap(VK_RIGHT, 1);
    sleep_ms(25);
    let Some(p2) = copy_selection().map(|t| norm(&t)) else { return Err(guess_dir(prefix)) };
    let before = prefix.chars().count();
    let after = p2.chars().count();
    let (dir, remaining) = if after < before {
        (Dir::Ltr, steps - 1)
    } else {
        if after > before {
            shift_tap(VK_LEFT, 1); // that step went the wrong way, undo it
        }
        (Dir::Rtl, steps)
    };
    shift_tap(if dir == Dir::Ltr { VK_RIGHT } else { VK_LEFT }, remaining);
    sleep_ms(30);
    match copy_selection().map(|t| norm(&t)) {
        Some(t) if t == norm(expected) => Ok(()),
        _ => Err(dir),
    }
}

/// Second press right after a conversion: put the original text back.
fn try_undo(s: &Settings) -> bool {
    let Some(u) = take_undo() else { return false };
    let out = norm(&u.out);
    let n = key_steps(&out);
    if n == 0 || n > 2000 {
        return false;
    }
    let selected = if out.contains('\n') {
        shift_tap(VK_LEFT, n);
        sleep_ms(30);
        if copy_selection().map(|t| norm(&t)).as_deref() == Some(out.as_str()) {
            true
        } else {
            collapse(Dir::Ltr);
            false
        }
    } else {
        match select_line_prefix() {
            Some(prefix) if prefix.ends_with(&out) => {
                let steps = key_steps(&prefix[..prefix.len() - out.len()]);
                match shrink_to(&prefix, steps, &out) {
                    Ok(()) => true,
                    Err(d) => {
                        collapse(d);
                        false
                    }
                }
            }
            Some(prefix) => {
                collapse(guess_dir(&prefix));
                false
            }
            None => false,
        }
    };
    if selected && paste_text(&u.orig) {
        if s.switch_lang {
            switch_layout_for(&u.orig);
        }
        return true;
    }
    false
}

fn fix_inner(s: &Settings) {
    let mut converted_from: Option<(String, String)> = None; // (original selection, text to paste)

    match copy_selection() {
        Some(sel) => {
            let trimmed = sel.trim_end();
            let (core, trail) = sel.split_at(trimmed.len());
            if core.is_empty() {
                return;
            }
            let out = format!("{}{}", convert(core, s.direction), trail);
            if out != sel {
                converted_from = Some((sel.clone(), out));
            }
        }
        None => {
            if try_undo(s) {
                return;
            }
            if !s.last_word {
                return;
            }
            let Some(prefix) = select_line_prefix() else { return };
            let Some((steps, unit, trail)) = last_unit(&prefix, s.last_mode) else {
                collapse(guess_dir(&prefix));
                return;
            };
            let orig = format!("{}{}", unit, trail);
            let dir = match shrink_to(&prefix, steps, &orig) {
                Ok(()) => guess_dir(&prefix),
                Err(d) => {
                    collapse(d);
                    return;
                }
            };
            let out = format!("{}{}", convert(&unit, s.direction), trail);
            if out == orig {
                collapse(dir);
                return;
            }
            converted_from = Some((orig, out));
        }
    }

    if let Some((orig, out)) = converted_from {
        if paste_text(&out) {
            remember_undo(&out, &orig);
            if s.switch_lang {
                switch_layout_for(&out);
            }
        }
    }
}

fn do_fix() {
    let s = settings();
    if !s.enabled {
        return;
    }
    if let Some(hk) = parse_hotkey(&s.hotkey) {
        wait_release(&hotkey_vks(hk.vk));
    }
    let saved = save_clipboard();
    fix_inner(&s);
    restore_clipboard(saved);
}

// ---------------------------------------------------------------------------
// Tray icon and menu
// ---------------------------------------------------------------------------

fn copy_wide<const N: usize>(dst: &mut [u16; N], s: &str) {
    for (i, u) in s.encode_utf16().take(N - 1).enumerate() {
        dst[i] = u;
    }
}

fn nid(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = 1;
    data
}

fn add_tray(hwnd: HWND) {
    unsafe {
        let hinst = GetModuleHandleW(null());
        let mut icon = LoadIconW(hinst, 1usize as *const u16); // resource id 1 = baddel.ico
        if icon.is_null() {
            icon = LoadIconW(null_mut(), IDI_APPLICATION);
        }
        let mut data = nid(hwnd);
        data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        data.uCallbackMessage = WM_TRAY;
        data.hIcon = icon;
        copy_wide(&mut data.szTip, "Baddel");
        Shell_NotifyIconW(NIM_ADD, &data);
    }
}

fn remove_tray(hwnd: HWND) {
    let data = nid(hwnd);
    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

fn balloon(title: &str, text: &str) {
    let mut data = nid(hwnd_main());
    data.uFlags = NIF_INFO;
    data.dwInfoFlags = NIIF_INFO;
    copy_wide(&mut data.szInfoTitle, title);
    copy_wide(&mut data.szInfo, text);
    unsafe {
        Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

fn flag(on: bool) -> u32 {
    if on {
        MF_STRING | MF_CHECKED
    } else {
        MF_STRING
    }
}

fn append(menu: windows_sys::Win32::UI::WindowsAndMessaging::HMENU, flags: u32, id: usize, text: &str) {
    let w = wide(text);
    unsafe {
        AppendMenuW(menu, flags, id, w.as_ptr());
    }
}

fn show_menu(hwnd: HWND) {
    let s = settings();
    let startup_on = startup_entry().is_some();
    unsafe {
        let menu = CreatePopupMenu();
        let dir = CreatePopupMenu();
        let keys = CreatePopupMenu();

        append(menu, MF_STRING | 0x1, 0, &format!("Baddel  ({})", s.hotkey)); // 0x1 = MF_GRAYED
        append(menu, MF_STRING, ID_SETTINGS, "Settings...");
        append(menu, MF_SEPARATOR, 0, "");
        append(menu, flag(s.enabled), ID_ENABLED, "Enabled");

        append(dir, flag(s.direction == Direction::Auto), ID_DIR_AUTO, "Auto (word by word)");
        append(dir, flag(s.direction == Direction::ArToEn), ID_DIR_AR2EN, "Arabic -> English");
        append(dir, flag(s.direction == Direction::EnToAr), ID_DIR_EN2AR, "English -> Arabic");
        let dir_title = wide("Direction");
        AppendMenuW(menu, MF_POPUP, dir as usize, dir_title.as_ptr());

        for (i, p) in PRESETS.iter().enumerate() {
            append(keys, flag(format_hotkey(p) == s.hotkey), ID_PRESET_BASE + i, p);
        }
        append(keys, MF_SEPARATOR, 0, "");
        append(keys, MF_STRING, ID_OPEN_FILE, "Custom: edit the settings file...");
        let keys_title = wide("Shortcut");
        AppendMenuW(menu, MF_POPUP, keys as usize, keys_title.as_ptr());

        append(menu, flag(s.last_word), ID_LASTWORD, "Convert the last word when nothing is selected");
        append(menu, flag(s.switch_lang), ID_SWITCHLANG, "Switch the keyboard language after converting");
        append(menu, flag(startup_on), ID_STARTUP, "Start with Windows");
        append(menu, MF_SEPARATOR, 0, "");
        append(menu, MF_STRING, ID_RELOAD, "Reload settings file");
        append(menu, MF_STRING, ID_WEBSITE, "Website");
        append(menu, MF_STRING, ID_EXIT, "Exit");

        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd); // required, otherwise the menu does not close when clicking elsewhere
        TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_BOTTOMALIGN, pt.x, pt.y, 0, hwnd, null());
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu); // also destroys the submenus
    }
}

fn open_shell(target: &str, params: Option<&str>) {
    let op = wide("open");
    let file = wide(target);
    let args = params.map(wide);
    unsafe {
        ShellExecuteW(
            null_mut(),
            op.as_ptr(),
            file.as_ptr(),
            args.as_ref().map(|a| a.as_ptr()).unwrap_or(null()),
            null(),
            1,
        );
    }
}

fn layout_list() -> Vec<HKL> {
    unsafe {
        let n = GetKeyboardLayoutList(0, null_mut());
        if n < 1 {
            return Vec::new();
        }
        let mut list: Vec<HKL> = vec![null_mut(); n as usize];
        let got = GetKeyboardLayoutList(n, list.as_mut_ptr());
        list.truncate(got.max(0) as usize);
        list
    }
}

/// The US English layout when it is installed, otherwise the first layout.
fn us_layout() -> HKL {
    let list = layout_list();
    list.iter()
        .copied()
        .find(|h| (*h as usize) & 0xFFFF == 0x0409)
        .or_else(|| list.first().copied())
        .unwrap_or(null_mut())
}

/// Shortcuts are stored as US layout keys. On other layouts (Arabic, ...) the same physical key
/// reports a different virtual-key code, so find every code that key can produce.
fn hotkey_vks(vk: u32) -> Vec<u32> {
    let mut out = vec![vk];
    let us = us_layout();
    if us.is_null() {
        return out;
    }
    unsafe {
        let sc = MapVirtualKeyExW(vk, 0, us); // VK -> scan code
        if sc == 0 {
            return out;
        }
        for hkl in layout_list() {
            let v = MapVirtualKeyExW(sc, 1, hkl); // scan code -> VK
            if v != 0 && !out.contains(&v) && out.len() < HOTKEY_SLOTS as usize {
                out.push(v);
            }
        }
    }
    out
}

fn layout_signature() -> Vec<usize> {
    layout_list().iter().map(|h| *h as usize).collect()
}

static LAYOUT_SIG: Mutex<Vec<usize>> = Mutex::new(Vec::new());

fn register_hotkey(hwnd: HWND) {
    register_hotkey_ex(hwnd, true);
}

fn register_hotkey_ex(hwnd: HWND, announce: bool) {
    let s = settings();
    unsafe {
        for i in 0..HOTKEY_SLOTS {
            UnregisterHotKey(hwnd, HOTKEY_ID + i);
        }
    }
    *LAYOUT_SIG.lock().unwrap() = layout_signature();
    match parse_hotkey(&s.hotkey) {
        Some(hk) => {
            let vks = hotkey_vks(hk.vk);
            let mut main_ok = false;
            for (i, vk) in vks.iter().enumerate() {
                let ok = unsafe { RegisterHotKey(hwnd, HOTKEY_ID + i as i32, hk.mods | MOD_NOREPEAT, *vk) };
                if i == 0 {
                    main_ok = ok != 0;
                }
            }
            if !main_ok && announce {
                balloon("Baddel", &format!("{} is already used by another program. Pick another shortcut in Settings.", s.hotkey));
            }
        }
        None => {
            if announce {
                balloon("Baddel", "The shortcut in the settings file is not valid.");
            }
        }
    }
}

fn update_settings<F: FnOnce(&mut Settings)>(f: F) {
    let updated = {
        let mut guard = APP.lock().unwrap();
        match guard.as_mut() {
            Some(app) => {
                f(&mut app.settings);
                app.settings.clone()
            }
            None => return,
        }
    };
    save_settings(&updated);
}

fn on_command(hwnd: HWND, id: usize) {
    match id {
        ID_SETTINGS => open_settings(),
        ID_ENABLED => update_settings(|s| s.enabled = !s.enabled),
        ID_LASTWORD => update_settings(|s| s.last_word = !s.last_word),
        ID_SWITCHLANG => update_settings(|s| s.switch_lang = !s.switch_lang),
        ID_STARTUP => {
            let now_on = startup_entry().is_some();
            if !set_startup(!now_on) {
                balloon("Baddel", "Could not change the Start with Windows setting.");
            }
        }
        ID_DIR_AUTO => update_settings(|s| s.direction = Direction::Auto),
        ID_DIR_AR2EN => update_settings(|s| s.direction = Direction::ArToEn),
        ID_DIR_EN2AR => update_settings(|s| s.direction = Direction::EnToAr),
        ID_OPEN_FILE => {
            save_settings(&settings());
            open_shell("notepad.exe", Some(&format!("\"{}\"", settings_path())));
        }
        ID_RELOAD => {
            let fresh = load_settings();
            if let Some(app) = APP.lock().unwrap().as_mut() {
                app.settings = fresh;
            }
            register_hotkey(hwnd);
            balloon("Baddel", &format!("Settings reloaded. Shortcut: {}", settings().hotkey));
        }
        ID_WEBSITE => open_shell(WEBSITE, None),
        ID_EXIT => unsafe {
            DestroyWindow(hwnd);
        },
        i if (ID_PRESET_BASE..ID_PRESET_BASE + PRESETS.len()).contains(&i) => {
            let preset = format_hotkey(PRESETS[i - ID_PRESET_BASE]);
            update_settings(|s| s.hotkey = preset.clone());
            register_hotkey(hwnd);
            balloon("Baddel", &format!("Shortcut is now {}", preset));
        }
        _ => {}
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            do_fix();
            0
        }
        WM_TIMER if wparam == TIMER_LAYOUTS => {
            // the user installed or removed a keyboard layout: refresh the shortcut keys
            if *LAYOUT_SIG.lock().unwrap() != layout_signature() {
                register_hotkey_ex(hwnd, false);
            }
            0
        }
        WM_TRAY => {
            let event = (lparam as u32) & 0xFFFF;
            if event == WM_LBUTTONDBLCLK {
                open_settings();
            } else if event == WM_RBUTTONUP || event == WM_CONTEXTMENU || event == WM_LBUTTONUP {
                show_menu(hwnd);
            }
            0
        }
        WM_COMMAND => {
            on_command(hwnd, wparam & 0xFFFF);
            0
        }
        WM_DESTROY => {
            remove_tray(hwnd);
            for i in 0..HOTKEY_SLOTS {
                UnregisterHotKey(hwnd, HOTKEY_ID + i);
            }
            KillTimer(hwnd, TIMER_LAYOUTS);
            PostQuitMessage(0);
            0
        }
        m if m != 0 && m == TASKBAR_CREATED.load(Ordering::Relaxed) => {
            add_tray(hwnd); // Explorer restarted: put the icon back
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}


// ---------------------------------------------------------------------------
// Settings window (plain Win32 controls)
// ---------------------------------------------------------------------------

static SETTINGS_HWND: AtomicUsize = AtomicUsize::new(0);
static SETTINGS_FONT: AtomicUsize = AtomicUsize::new(0);

const C_ENABLED: usize = 2001;
const C_HOTKEY: usize = 2002;
const C_HK_STATUS: usize = 2003;
const C_DIR_AUTO: usize = 2010;
const C_DIR_AR2EN: usize = 2011;
const C_DIR_EN2AR: usize = 2012;
const C_LAST: usize = 2020;
const C_LAST_WORD: usize = 2021;
const C_LAST_RUN: usize = 2022;
const C_SWITCH: usize = 2030;
const C_STARTUP: usize = 2031;
const C_TRY_IN: usize = 2040;
const C_TRY_OUT: usize = 2041;
const EN_CHANGE: usize = 0x0300;
const BN_CLICKED: usize = 0;

struct Ui {
    hwnd: HWND,
    font: usize,
    dpi: i32,
}

impl Ui {
    fn sc(&self, v: i32) -> i32 {
        v * self.dpi / 96
    }
    #[allow(clippy::too_many_arguments)]
    fn add(&self, class: &str, text: &str, style: u32, ex: u32, x: i32, y: i32, w: i32, h: i32, id: usize) -> HWND {
        unsafe {
            let c = wide(class);
            let t = wide(text);
            let h_ctl = CreateWindowExW(
                ex,
                c.as_ptr(),
                t.as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                self.sc(x),
                self.sc(y),
                self.sc(w),
                self.sc(h),
                self.hwnd,
                id as _,
                GetModuleHandleW(null()),
                null(),
            );
            SendMessageW(h_ctl, WM_SETFONT, self.font, 1);
            h_ctl
        }
    }
    fn label(&self, text: &str, x: i32, y: i32, w: i32, h: i32, id: usize) {
        self.add("STATIC", text, 0, 0, x, y, w, h, id);
    }
    fn check(&self, text: &str, x: i32, y: i32, w: i32, id: usize) {
        self.add("BUTTON", text, BS_AUTOCHECKBOX as u32 | WS_TABSTOP, 0, x, y, w, 20, id);
    }
    fn radio(&self, text: &str, x: i32, y: i32, w: i32, id: usize, first: bool) {
        let g = if first { WS_GROUP | WS_TABSTOP } else { 0 };
        self.add("BUTTON", text, BS_AUTORADIOBUTTON as u32 | g, 0, x, y, w, 20, id);
    }
}

fn dlg_text(hwnd: HWND, id: usize) -> String {
    let mut buf = vec![0u16; 4096];
    let n = unsafe { GetDlgItemTextW(hwnd, id as i32, buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..n as usize])
}

fn set_dlg_text(hwnd: HWND, id: usize, text: &str) {
    let w = wide(text);
    unsafe {
        SetDlgItemTextW(hwnd, id as i32, w.as_ptr());
    }
}

fn checked(hwnd: HWND, id: usize) -> bool {
    unsafe { IsDlgButtonChecked(hwnd, id as i32) == 1 }
}

fn set_checked(hwnd: HWND, id: usize, on: bool) {
    unsafe {
        CheckDlgButton(hwnd, id as i32, if on { 1 } else { 0 });
    }
}

fn dialog_direction(hwnd: HWND) -> Direction {
    if checked(hwnd, C_DIR_AR2EN) {
        Direction::ArToEn
    } else if checked(hwnd, C_DIR_EN2AR) {
        Direction::EnToAr
    } else {
        Direction::Auto
    }
}

fn refresh_try(hwnd: HWND) {
    let out = convert(&dlg_text(hwnd, C_TRY_IN), dialog_direction(hwnd));
    set_dlg_text(hwnd, C_TRY_OUT, &out);
}

fn refresh_hotkey(hwnd: HWND) {
    let text = dlg_text(hwnd, C_HOTKEY);
    let msg = match parse_hotkey(&text) {
        Some(hk) => format!("OK  {}", hotkey_text(hk.mods, hk.vk).unwrap_or_else(|| format_hotkey(&text))),
        None => "Click the box and press the keys".to_string(),
    };
    set_dlg_text(hwnd, C_HK_STATUS, &msg);
}

fn refresh_last_mode(hwnd: HWND) {
    let on = checked(hwnd, C_LAST);
    unsafe {
        EnableWindow(GetDlgItem(hwnd, C_LAST_WORD as i32), on as i32);
        EnableWindow(GetDlgItem(hwnd, C_LAST_RUN as i32), on as i32);
    }
}

fn load_into_dialog(hwnd: HWND, s: &Settings) {
    set_checked(hwnd, C_ENABLED, s.enabled);
    set_dlg_text(hwnd, C_HOTKEY, &s.hotkey);
    unsafe {
        let id = match s.direction {
            Direction::Auto => C_DIR_AUTO,
            Direction::ArToEn => C_DIR_AR2EN,
            Direction::EnToAr => C_DIR_EN2AR,
        };
        CheckRadioButton(hwnd, C_DIR_AUTO as i32, C_DIR_EN2AR as i32, id as i32);
        let lm = if s.last_mode == LastMode::Word { C_LAST_WORD } else { C_LAST_RUN };
        CheckRadioButton(hwnd, C_LAST_WORD as i32, C_LAST_RUN as i32, lm as i32);
    }
    set_checked(hwnd, C_LAST, s.last_word);
    set_checked(hwnd, C_SWITCH, s.switch_lang);
    set_checked(hwnd, C_STARTUP, startup_entry().is_some());
    set_dlg_text(hwnd, C_TRY_IN, "hgsghl ugd;l");
    refresh_hotkey(hwnd);
    refresh_last_mode(hwnd);
    refresh_try(hwnd);
}

fn save_from_dialog(hwnd: HWND) -> bool {
    let hk_text = dlg_text(hwnd, C_HOTKEY);
    if parse_hotkey(&hk_text).is_none() {
        let t = wide("The shortcut is not valid.\nUse modifiers plus a key, for example Ctrl+Shift+X.");
        let c = wide("Baddel");
        unsafe {
            MessageBoxW(hwnd, t.as_ptr(), c.as_ptr(), MB_OK | MB_ICONWARNING);
        }
        return false;
    }
    let direction = dialog_direction(hwnd);
    let enabled = checked(hwnd, C_ENABLED);
    let last = checked(hwnd, C_LAST);
    let mode = if checked(hwnd, C_LAST_WORD) { LastMode::Word } else { LastMode::Run };
    let switch = checked(hwnd, C_SWITCH);
    let hotkey = format_hotkey(&hk_text);
    update_settings(|s| {
        s.enabled = enabled;
        s.direction = direction;
        s.last_word = last;
        s.last_mode = mode;
        s.switch_lang = switch;
        s.hotkey = hotkey.clone();
    });
    let want_startup = checked(hwnd, C_STARTUP);
    if want_startup != startup_entry().is_some() && !set_startup(want_startup) {
        balloon("Baddel", "Could not change the Start with Windows setting.");
    }
    register_hotkey(hwnd_main());
    true
}

fn build_settings_window(hwnd: HWND) {
    let dpi = unsafe { GetDpiForSystem() } as i32;
    let face = wide("Segoe UI");
    let font = unsafe { CreateFontW(-(9 * dpi / 72), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, face.as_ptr()) } as usize;
    SETTINGS_FONT.store(font, Ordering::Relaxed);
    let ui = Ui { hwnd, font, dpi };

    ui.check("Enabled", 16, 12, 200, C_ENABLED);
    ui.label("Shortcut:", 16, 44, 70, 20, 0);
    let hk = ui.add("EDIT", "", ES_AUTOHSCROLL as u32 | WS_TABSTOP, WS_EX_CLIENTEDGE, 90, 40, 170, 24, C_HOTKEY);
    unsafe {
        SetWindowSubclass(hk, Some(hotkey_capture), 1, 0);
    }
    ui.label("", 270, 44, 200, 20, C_HK_STATUS);

    ui.add("BUTTON", "Direction", BS_GROUPBOX as u32, 0, 12, 74, 456, 50, 0);
    ui.radio("Auto (word by word)", 24, 94, 160, C_DIR_AUTO, true);
    ui.radio("Arabic -> English", 190, 94, 130, C_DIR_AR2EN, false);
    ui.radio("English -> Arabic", 326, 94, 130, C_DIR_EN2AR, false);

    ui.check("Convert the text before the caret when nothing is selected", 16, 136, 440, C_LAST);
    ui.radio("Last word only", 40, 160, 150, C_LAST_WORD, true);
    ui.radio("Last words in the same language", 196, 160, 260, C_LAST_RUN, false);

    ui.check("Switch the keyboard language after converting", 16, 190, 440, C_SWITCH);
    ui.check("Start with Windows", 16, 214, 440, C_STARTUP);

    ui.add("BUTTON", "Try it", BS_GROUPBOX as u32, 0, 12, 244, 456, 92, 0);
    ui.add("EDIT", "", ES_AUTOHSCROLL as u32 | WS_TABSTOP, WS_EX_CLIENTEDGE, 24, 264, 432, 24, C_TRY_IN);
    ui.add("EDIT", "", ES_AUTOHSCROLL as u32 | ES_READONLY as u32, WS_EX_CLIENTEDGE, 24, 298, 432, 24, C_TRY_OUT);

    ui.add("BUTTON", "Save", BS_DEFPUSHBUTTON as u32 | WS_TABSTOP, 0, 292, 350, 84, 28, IDOK as usize);
    ui.add("BUTTON", "Cancel", BS_PUSHBUTTON as u32 | WS_TABSTOP, 0, 384, 350, 84, 28, IDCANCEL as usize);

    load_into_dialog(hwnd, &settings());
}

/// The shortcut box records the keys you press instead of taking typed text.
unsafe extern "system" fn hotkey_capture(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    match msg {
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let mut vk = wparam as u32;
            if !matches!(vk, 0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0xA0..=0xA5) {
                // other layouts report other codes for the same physical key: use the US meaning
                let sc = ((lparam >> 16) & 0xFF) as u32;
                let us = us_layout();
                if sc != 0 && !us.is_null() {
                    let v = MapVirtualKeyExW(sc, 1, us);
                    if v != 0 {
                        vk = v;
                    }
                }
            }
            let mut mods = 0u32;
            if is_down(VK_CONTROL as i32) {
                mods |= 2;
            }
            if is_down(VK_MENU as i32) {
                mods |= 1;
            }
            if is_down(VK_SHIFT as i32) {
                mods |= 4;
            }
            if is_down(VK_LWIN as i32) || is_down(VK_RWIN as i32) {
                mods |= 8;
            }
            if matches!(vk, 0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0xA0..=0xA5) {
                // only modifiers so far: show them, waiting for the main key
                let mut parts: Vec<&str> = Vec::new();
                if mods & 2 != 0 {
                    parts.push("Ctrl");
                }
                if mods & 1 != 0 {
                    parts.push("Alt");
                }
                if mods & 4 != 0 {
                    parts.push("Shift");
                }
                if mods & 8 != 0 {
                    parts.push("Win");
                }
                let w = wide(&format!("{}+", parts.join("+")));
                SetWindowTextW(hwnd, w.as_ptr());
            } else if let Some(text) = hotkey_text(mods, vk) {
                let w = wide(&text);
                SetWindowTextW(hwnd, w.as_ptr());
            } else if mods == 0 && (0x70..=0x87).contains(&vk) {
                let w = wide(&format!("F{}", vk - 0x6F));
                SetWindowTextW(hwnd, w.as_ptr());
            }
            0
        }
        WM_CHAR | WM_SYSCHAR => 0,
        _ => DefSubclassProc(hwnd, msg, wparam, lparam),
    }
}

unsafe extern "system" fn settings_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            let code = (wparam >> 16) & 0xFFFF;
            match id {
                x if x == IDOK as usize => {
                    if save_from_dialog(hwnd) {
                        DestroyWindow(hwnd);
                    }
                }
                x if x == IDCANCEL as usize => {
                    DestroyWindow(hwnd);
                }
                C_HOTKEY if code == EN_CHANGE => refresh_hotkey(hwnd),
                C_TRY_IN if code == EN_CHANGE => refresh_try(hwnd),
                C_DIR_AUTO | C_DIR_AR2EN | C_DIR_EN2AR if code == BN_CLICKED => refresh_try(hwnd),
                C_LAST if code == BN_CLICKED => refresh_last_mode(hwnd),
                _ => {}
            }
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            SETTINGS_HWND.store(0, Ordering::Relaxed);
            let f = SETTINGS_FONT.swap(0, Ordering::Relaxed);
            if f != 0 {
                DeleteObject(f as _);
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn open_settings() {
    let existing = SETTINGS_HWND.load(Ordering::Relaxed) as HWND;
    if !existing.is_null() {
        unsafe {
            SetForegroundWindow(existing);
        }
        return;
    }
    unsafe {
        let hinst = GetModuleHandleW(null());
        let class_name = wide("BaddelSettingsWindow");
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(settings_proc);
        wc.hInstance = hinst;
        wc.lpszClassName = class_name.as_ptr();
        wc.hbrBackground = GetSysColorBrush(COLOR_BTNFACE);
        wc.hIcon = LoadIconW(hinst, 1usize as *const u16);
        RegisterClassW(&wc); // fails harmlessly when the class already exists

        let dpi = GetDpiForSystem() as i32;
        let style = WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
        let mut rc = RECT { left: 0, top: 0, right: 480 * dpi / 96, bottom: 392 * dpi / 96 };
        AdjustWindowRectEx(&mut rc, style, 0, 0);
        let (w, h) = (rc.right - rc.left, rc.bottom - rc.top);
        let x = (GetSystemMetrics(SM_CXSCREEN) - w) / 2;
        let y = (GetSystemMetrics(SM_CYSCREEN) - h) / 3;
        let title = wide("Baddel - Settings");
        let hwnd = CreateWindowExW(0, class_name.as_ptr(), title.as_ptr(), style, x, y, w, h, null_mut(), null_mut(), hinst, null());
        if hwnd.is_null() {
            return;
        }
        SETTINGS_HWND.store(hwnd as usize, Ordering::Relaxed);
        build_settings_window(hwnd);
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
    }
}

pub fn run() {
    unsafe {
        // one copy only
        let name = wide("Baddel.SingleInstance.Mutex");
        let _mutex = CreateMutexW(null(), 0, name.as_ptr());
        if GetLastError() == 183 {
            return; // ERROR_ALREADY_EXISTS
        }

        let settings = load_settings();
        *APP.lock().unwrap() = Some(App { settings, undo: None });

        // keep the Start with Windows entry pointing at the file that is running now
        if let Some(cmd) = startup_entry() {
            if !cmd.eq_ignore_ascii_case(&exe_command()) {
                set_startup(true);
            }
        }

        let hinst = GetModuleHandleW(null());
        let class_name = wide("BaddelHiddenWindow");
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(wndproc);
        wc.hInstance = hinst;
        wc.lpszClassName = class_name.as_ptr();
        RegisterClassW(&wc);

        TASKBAR_CREATED.store(RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()), Ordering::Relaxed);

        // a normal (never shown) window, because message-only windows miss "TaskbarCreated"
        let title = wide("Baddel");
        let hwnd = CreateWindowExW(0, class_name.as_ptr(), title.as_ptr(), 0, 0, 0, 0, 0, null_mut(), null_mut(), hinst, null());
        if hwnd.is_null() {
            return;
        }
        *HWND_MAIN.lock().unwrap() = hwnd as usize;

        add_tray(hwnd);
        register_hotkey(hwnd);
        SetTimer(hwnd, TIMER_LAYOUTS, 5000, None);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            let dlg = SETTINGS_HWND.load(Ordering::Relaxed) as HWND;
            if !dlg.is_null() && IsDialogMessageW(dlg, &msg) != 0 {
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    let _ = DEFAULT_HOTKEY;
}

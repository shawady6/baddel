//! Platform independent logic: conversion, settings and shortcut parsing.
//! Everything here is unit-tested on any OS; `main.rs` adds the Windows glue.

use crate::tables::{AR_TO_EN, EN_TO_AR};

// ---------------------------------------------------------------------------
// Conversion
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    /// Decide word by word from each word's own script.
    Auto,
    ArToEn,
    EnToAr,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Auto => "auto",
            Direction::ArToEn => "ar2en",
            Direction::EnToAr => "en2ar",
        }
    }
    pub fn parse(s: &str) -> Direction {
        match s.trim().to_ascii_lowercase().as_str() {
            "ar2en" => Direction::ArToEn,
            "en2ar" => Direction::EnToAr,
            _ => Direction::Auto,
        }
    }
}

fn is_arabic(c: char) -> bool {
    ('\u{0600}'..='\u{06FF}').contains(&c)
}

fn is_latin(c: char) -> bool {
    c.is_ascii_alphabetic()
}

fn map_text(text: &str, table: &[(char, &str)]) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match table.iter().find(|(k, _)| *k == ch) {
            Some((_, v)) => out.push_str(v),
            None => out.push(ch),
        }
    }
    out
}

fn convert_word(word: &str) -> String {
    let ar = word.chars().filter(|c| is_arabic(*c)).count();
    let la = word.chars().filter(|c| is_latin(*c)).count();
    if ar == 0 && la == 0 {
        return word.to_string();
    }
    map_text(word, if ar >= la { AR_TO_EN } else { EN_TO_AR })
}

pub fn convert(text: &str, dir: Direction) -> String {
    match dir {
        Direction::ArToEn => map_text(text, AR_TO_EN),
        Direction::EnToAr => map_text(text, EN_TO_AR),
        Direction::Auto => {
            // keep every whitespace run exactly, convert the words in between
            let mut out = String::with_capacity(text.len());
            let mut word = String::new();
            for ch in text.chars() {
                if ch.is_whitespace() {
                    if !word.is_empty() {
                        out.push_str(&convert_word(&word));
                        word.clear();
                    }
                    out.push(ch);
                } else {
                    word.push(ch);
                }
            }
            if !word.is_empty() {
                out.push_str(&convert_word(&word));
            }
            out
        }
    }
}

/// Which script a piece of text is mostly written in (used to pick the keyboard layout).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Script {
    Arabic,
    Latin,
}

pub fn dominant_script(text: &str) -> Option<Script> {
    let ar = text.chars().filter(|c| is_arabic(*c)).count();
    let la = text.chars().filter(|c| is_latin(*c)).count();
    if ar == 0 && la == 0 {
        None
    } else if ar >= la {
        Some(Script::Arabic)
    } else {
        Some(Script::Latin)
    }
}

// ---------------------------------------------------------------------------
// "Last word" helper
// ---------------------------------------------------------------------------

/// Number of key presses a caret needs to step over `s`
/// (a CRLF pair is a single press).
pub fn key_steps(s: &str) -> usize {
    let mut n = 0;
    let mut prev_cr = false;
    for ch in s.chars() {
        if ch == '\n' && prev_cr {
            prev_cr = false;
            continue;
        }
        prev_cr = ch == '\r';
        n += 1;
    }
    n
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LastMode {
    /// Only the word right before the caret.
    Word,
    /// The run of words before the caret that are written in the same script.
    Run,
}

impl LastMode {
    pub fn as_str(self) -> &'static str {
        match self {
            LastMode::Word => "word",
            LastMode::Run => "run",
        }
    }
    pub fn parse(s: &str) -> LastMode {
        match s.trim().to_ascii_lowercase().as_str() {
            "word" => LastMode::Word,
            _ => LastMode::Run,
        }
    }
}

/// `prefix` is the text from the start of the line up to the caret. Picks what to convert
/// when nothing is selected. Returns `(steps_to_unselect, unit, trailing_whitespace)`:
/// `steps_to_unselect` is how many characters of `prefix` stay unconverted.
pub fn last_unit(prefix: &str, mode: LastMode) -> Option<(usize, String, String)> {
    let core_end = prefix.trim_end().len();
    let (core, trail) = prefix.split_at(core_end);
    if core.is_empty() {
        return None;
    }
    // (byte offset, token) for every whitespace separated token
    let mut tokens: Vec<(usize, &str)> = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in core.char_indices() {
        if c.is_whitespace() {
            if let Some(st) = start.take() {
                tokens.push((st, &core[st..i]));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(st) = start {
        tokens.push((st, &core[st..]));
    }
    let (mut first, last) = (tokens.len() - 1, tokens[tokens.len() - 1].1);
    if mode == LastMode::Run {
        if let Some(script) = dominant_script(last) {
            while first > 0 && dominant_script(tokens[first - 1].1) == Some(script) {
                first -= 1;
            }
        }
    }
    let offset = tokens[first].0;
    Some((key_steps(&core[..offset]), core[offset..].to_string(), trail.to_string()))
}

/// Reverse of `key_to_vk` for the keys we can show in the settings window.
pub fn vk_name(vk: u32) -> Option<String> {
    match vk {
        0x41..=0x5A => Some(((vk as u8) as char).to_string()),
        0x30..=0x39 => Some(((vk as u8) as char).to_string()),
        0x70..=0x87 => Some(format!("F{}", vk - 0x6F)),
        0xBA => Some(";".into()),
        0xBB => Some("=".into()),
        0xBC => Some(",".into()),
        0xBD => Some("-".into()),
        0xBE => Some(".".into()),
        0xBF => Some("/".into()),
        0xC0 => Some("`".into()),
        0xDB => Some("[".into()),
        0xDC => Some("\\".into()),
        0xDD => Some("]".into()),
        0xDE => Some("'".into()),
        0x20 => Some("Space".into()),
        0x0D => Some("Enter".into()),
        0x09 => Some("Tab".into()),
        0x1B => Some("Esc".into()),
        0x08 => Some("Backspace".into()),
        0x2E => Some("Delete".into()),
        0x2D => Some("Insert".into()),
        0x24 => Some("Home".into()),
        0x23 => Some("End".into()),
        0x21 => Some("PageUp".into()),
        0x22 => Some("PageDown".into()),
        0x26 => Some("Up".into()),
        0x28 => Some("Down".into()),
        0x25 => Some("Left".into()),
        0x27 => Some("Right".into()),
        _ => None,
    }
}

pub fn hotkey_text(mods: u32, vk: u32) -> Option<String> {
    let key = vk_name(vk)?;
    let mut parts: Vec<&str> = Vec::new();
    if mods & MOD_CONTROL != 0 {
        parts.push("Ctrl");
    }
    if mods & MOD_ALT != 0 {
        parts.push("Alt");
    }
    if mods & MOD_SHIFT != 0 {
        parts.push("Shift");
    }
    if mods & MOD_WIN != 0 {
        parts.push("Win");
    }
    if parts.is_empty() {
        return None;
    }
    Some(format!("{}+{}", parts.join("+"), key))
}

// ---------------------------------------------------------------------------
// Shortcut parsing
// ---------------------------------------------------------------------------

pub const MOD_ALT: u32 = 0x1;
pub const MOD_CONTROL: u32 = 0x2;
pub const MOD_SHIFT: u32 = 0x4;
pub const MOD_WIN: u32 = 0x8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hotkey {
    pub mods: u32,
    pub vk: u32,
}

fn key_to_vk(name: &str) -> Option<u32> {
    let n = name.to_ascii_lowercase();
    let b = n.as_bytes();
    if b.len() == 1 {
        let c = b[0];
        return match c {
            b'a'..=b'z' => Some(0x41 + (c - b'a') as u32),
            b'0'..=b'9' => Some(0x30 + (c - b'0') as u32),
            b';' => Some(0xBA),
            b'=' => Some(0xBB),
            b',' => Some(0xBC),
            b'-' => Some(0xBD),
            b'.' => Some(0xBE),
            b'/' => Some(0xBF),
            b'`' => Some(0xC0),
            b'[' => Some(0xDB),
            b'\\' => Some(0xDC),
            b']' => Some(0xDD),
            b'\'' => Some(0xDE),
            _ => None,
        };
    }
    if let Some(num) = n.strip_prefix('f') {
        if let Ok(k) = num.parse::<u32>() {
            if (1..=24).contains(&k) {
                return Some(0x6F + k);
            }
        }
    }
    match n.as_str() {
        "space" => Some(0x20),
        "enter" | "return" => Some(0x0D),
        "tab" => Some(0x09),
        "esc" | "escape" => Some(0x1B),
        "backspace" => Some(0x08),
        "delete" | "del" => Some(0x2E),
        "insert" | "ins" => Some(0x2D),
        "home" => Some(0x24),
        "end" => Some(0x23),
        "pageup" | "pgup" => Some(0x21),
        "pagedown" | "pgdn" => Some(0x22),
        "up" => Some(0x26),
        "down" => Some(0x28),
        "left" => Some(0x25),
        "right" => Some(0x27),
        _ => None,
    }
}

/// "Ctrl+Shift+X" -> Hotkey. Needs at least one modifier and exactly one key.
pub fn parse_hotkey(text: &str) -> Option<Hotkey> {
    let mut mods = 0;
    let mut key: Option<u32> = None;
    // split on '+' but allow a literal '+' key written as "Ctrl++"
    let text = text.trim();
    let (head, tail) = match text.strip_suffix("++") {
        Some(h) => (h, Some("+")),
        None => (text, None),
    };
    if tail.is_some() {
        return None; // '+' as the main key is not supported
    }
    for part in head.split('+') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        match p.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => mods |= MOD_CONTROL,
            "shift" => mods |= MOD_SHIFT,
            "alt" => mods |= MOD_ALT,
            "win" | "windows" | "meta" | "super" => mods |= MOD_WIN,
            _ => {
                if key.is_some() {
                    return None; // only one main key
                }
                key = Some(key_to_vk(p)?);
            }
        }
    }
    match (mods, key) {
        (0, _) | (_, None) => None,
        (m, Some(vk)) => Some(Hotkey { mods: m, vk }),
    }
}

/// Normalised display form, e.g. "Ctrl+Shift+X".
pub fn format_hotkey(text: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut key = String::new();
    for part in text.split('+') {
        let p = part.trim();
        match p.to_ascii_lowercase().as_str() {
            "" => {}
            "ctrl" | "control" => parts.push("Ctrl".into()),
            "shift" => parts.push("Shift".into()),
            "alt" => parts.push("Alt".into()),
            "win" | "windows" | "meta" | "super" => parts.push("Win".into()),
            _ => key = if p.len() == 1 { p.to_ascii_uppercase() } else { p.to_string() },
        }
    }
    let order = ["Ctrl", "Alt", "Shift", "Win"];
    let mut sorted: Vec<String> = Vec::new();
    for o in order {
        if parts.iter().any(|p| p == o) {
            sorted.push(o.to_string());
        }
    }
    sorted.push(key);
    sorted.join("+")
}

// ---------------------------------------------------------------------------
// Settings (a tiny key=value file)
// ---------------------------------------------------------------------------

pub const SNIPPETS_HOTKEY: &str = "Ctrl+Alt+E";
pub const DIGITS_HOTKEY: &str = "Ctrl+Alt+H";
pub const DEFAULT_HOTKEY: &str = "Ctrl+Shift+X";

/// Western digits become Arabic-Indic digits (0123 -> \u{660}\u{661}\u{662}\u{663}). When the text has
/// no Western digit but has Arabic-Indic (or Persian) ones, they go back to Western.
pub fn convert_digits(text: &str) -> String {
    let has_western = text.chars().any(|c| c.is_ascii_digit());
    text.chars()
        .map(|c| {
            if has_western {
                match c {
                    '0'..='9' => char::from_u32(0x0660 + (c as u32 - '0' as u32)).unwrap_or(c),
                    _ => c,
                }
            } else {
                match c {
                    '\u{0660}'..='\u{0669}' => char::from_u32('0' as u32 + (c as u32 - 0x0660)).unwrap_or(c),
                    '\u{06F0}'..='\u{06F9}' => char::from_u32('0' as u32 + (c as u32 - 0x06F0)).unwrap_or(c),
                    _ => c,
                }
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Snippets: type a trigger, press the shortcut, get the text
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snippet {
    pub trigger: String,
    pub text: String,
}

fn escape_snippet(text: &str) -> String {
    text.replace('\\', "\\\\").replace("\r\n", "\n").replace('\n', "\\n")
}

fn unescape_snippet(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('\\') => out.push('\\'),
                Some(o) => {
                    out.push('\\');
                    out.push(o);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// File format: one `trigger=text` per line, `\n` inside the text means a new line.
pub fn parse_snippets(file: &str) -> Vec<Snippet> {
    let mut list: Vec<Snippet> = Vec::new();
    for line in file.lines() {
        let line = line.trim_start_matches('\u{feff}');
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let Some((t, body)) = line.split_once('=') else { continue };
        let trigger = t.trim().to_string();
        if trigger.is_empty() || list.iter().any(|x| x.trigger.to_lowercase() == trigger.to_lowercase()) {
            continue;
        }
        list.push(Snippet { trigger, text: unescape_snippet(body) });
    }
    list
}

pub fn serialize_snippets(list: &[Snippet]) -> String {
    let mut out = String::from(
        "# Baddel snippets. One per line:  trigger=text   (use \\n for a new line; {date} and {time} are filled in)\r\n",
    );
    for sn in list {
        out.push_str(&format!("{}={}\r\n", sn.trigger.replace('=', " "), escape_snippet(&sn.text)));
    }
    out
}

fn words_joined(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Index of the snippet whose trigger is `text` (ignoring case and spacing). `text` may also be the
/// trigger typed with the wrong keyboard language.
pub fn lookup_snippet(text: &str, list: &[Snippet]) -> Option<usize> {
    let a = words_joined(text);
    if a.is_empty() {
        return None;
    }
    let flipped = words_joined(&convert(text, Direction::Auto));
    list.iter()
        .position(|sn| words_joined(&sn.trigger) == a)
        .or_else(|| list.iter().position(|sn| words_joined(&sn.trigger) == flipped))
}

/// The snippet trigger that ends `prefix` (the text from the line start to the caret), preferring
/// the longest one. Returns `(steps_before_trigger, trigger_text_as_typed, trailing_whitespace)`.
pub fn find_snippet(prefix: &str, list: &[Snippet]) -> Option<(usize, String, String)> {
    let core_end = prefix.trim_end().len();
    let (core, trail) = prefix.split_at(core_end);
    if core.is_empty() || list.is_empty() {
        return None;
    }
    let mut tokens: Vec<usize> = Vec::new(); // byte offset of each word
    let mut in_word = false;
    for (i, c) in core.char_indices() {
        if c.is_whitespace() {
            in_word = false;
        } else if !in_word {
            in_word = true;
            tokens.push(i);
        }
    }
    let max_words = list.iter().map(|sn| sn.trigger.split_whitespace().count()).max().unwrap_or(1);
    for n in (1..=max_words.min(tokens.len())).rev() {
        let start = tokens[tokens.len() - n];
        let tail = &core[start..];
        if lookup_snippet(tail, list).is_some() {
            return Some((key_steps(&core[..start]), tail.to_string(), trail.to_string()));
        }
    }
    None
}

/// Fill `{date}` and `{time}` into a snippet's text.
pub fn expand_placeholders(text: &str, date: &str, time: &str) -> String {
    text.replace("{date}", date).replace("{time}", time)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub enabled: bool,
    pub direction: Direction,
    pub last_word: bool,
    pub last_mode: LastMode,
    pub switch_lang: bool,
    pub hotkey: String,
    /// Optional second shortcut: Western digits <-> Arabic-Indic digits. Empty = off.
    pub digits_hotkey: String,
    /// Optional third shortcut: expand a snippet trigger into its text. Empty = off.
    pub snippets_hotkey: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            enabled: true,
            direction: Direction::Auto,
            last_word: true,
            last_mode: LastMode::Run,
            switch_lang: false,
            hotkey: DEFAULT_HOTKEY.to_string(),
            digits_hotkey: DIGITS_HOTKEY.to_string(),
            snippets_hotkey: SNIPPETS_HOTKEY.to_string(),
        }
    }
}

fn parse_bool(v: &str, default: bool) -> bool {
    match v.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default,
    }
}

impl Settings {
    pub fn parse(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            let line = line.trim().trim_start_matches('\u{feff}');
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else { continue };
            let v = v.trim();
            match k.trim().to_ascii_lowercase().as_str() {
                "enabled" => s.enabled = parse_bool(v, s.enabled),
                "direction" => s.direction = Direction::parse(v),
                "last_word" => s.last_word = parse_bool(v, s.last_word),
                "last_mode" => s.last_mode = LastMode::parse(v),
                "switch_language" => s.switch_lang = parse_bool(v, s.switch_lang),
                "hotkey" => {
                    if parse_hotkey(v).is_some() {
                        s.hotkey = format_hotkey(v);
                    }
                }
                "snippets_hotkey" => {
                    if v.is_empty() {
                        s.snippets_hotkey = String::new();
                    } else if parse_hotkey(v).is_some() {
                        s.snippets_hotkey = format_hotkey(v);
                    }
                }
                "digits_hotkey" => {
                    if v.is_empty() {
                        s.digits_hotkey = String::new();
                    } else if parse_hotkey(v).is_some() {
                        s.digits_hotkey = format_hotkey(v);
                    }
                }
                _ => {}
            }
        }
        s
    }

    pub fn serialize(&self) -> String {
        let b = |v: bool| if v { "1" } else { "0" };
        format!(
            "# Baddel settings. Edit and use the tray menu > Reload settings.\r\n\
             # hotkey: any mix of Ctrl, Alt, Shift, Win plus one key, e.g. Ctrl+Alt+Shift+K\r\n\
             hotkey={}\r\n\
             # direction: auto (word by word), ar2en or en2ar\r\n\
             direction={}\r\n\
             enabled={}\r\n\
             last_word={}\r\n\
             # last_mode: run (the words before the caret in the same language) or word (just one)\r\n\
             last_mode={}\r\n\
             switch_language={}\r\n\
             # digits_hotkey: Western digits <-> Arabic-Indic digits (0123 <-> \u{660}\u{661}\u{662}\u{663}). Leave empty to turn it off.\r\n\
             digits_hotkey={}\r\n\
             # snippets_hotkey: type a trigger word and press it to get its text (edit the list in Settings). Leave empty to turn it off.\r\n\
             snippets_hotkey={}\r\n",
            self.hotkey,
            self.direction.as_str(),
            b(self.enabled),
            b(self.last_word),
            self.last_mode.as_str(),
            b(self.switch_lang),
            self.digits_hotkey,
            self.snippets_hotkey,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_is_word_by_word() {
        assert_eq!(convert("hgsghl ugd;l", Direction::Auto), "السلام عليكم");
        assert_eq!(convert("lvpfh", Direction::Auto), "مرحبا");
        assert_eq!(convert("اثممخ", Direction::Auto), "hello");
        assert_eq!(convert("hgsghl ugd;l اثممخ", Direction::Auto), "السلام عليكم hello");
        assert_eq!(convert("اثممخ ,gpfh lvpfh", Direction::Auto), "hello ولحبا مرحبا");
    }

    #[test]
    fn auto_keeps_whitespace_digits_and_punctuation() {
        assert_eq!(convert("a  b\nhgsghl 123 !", Direction::Auto), "ش  لا\nالسلام 123 !");
        assert_eq!(convert("  ", Direction::Auto), "  ");
        assert_eq!(convert("", Direction::Auto), "");
    }

    #[test]
    fn forced_directions() {
        assert_eq!(convert("abc", Direction::ArToEn), "abc");
        assert_eq!(convert("hgsghl ugd;l", Direction::EnToAr), "السلام عليكم");
        assert_eq!(convert("اثممخ", Direction::ArToEn), "hello");
    }

    #[test]
    fn dominant() {
        assert_eq!(dominant_script("hello"), Some(Script::Latin));
        assert_eq!(dominant_script("مرحبا"), Some(Script::Arabic));
        assert_eq!(dominant_script("123 !"), None);
    }

    #[test]
    fn last_unit_after_line_break() {
        let (steps, unit, _) = last_unit("a\r\nbc", LastMode::Word).unwrap();
        assert_eq!((steps, unit.as_str()), (2, "bc"));
    }

    #[test]
    fn crlf_counts_once() {
        assert_eq!(key_steps("a\r\nb"), 3);
        assert_eq!(key_steps("\r\n"), 1);
        assert_eq!(key_steps("\n\n"), 2);
    }

    #[test]
    fn hotkeys() {
        assert_eq!(parse_hotkey("Ctrl+Shift+X"), Some(Hotkey { mods: MOD_CONTROL | MOD_SHIFT, vk: 0x58 }));
        assert_eq!(
            parse_hotkey("ctrl+alt+shift+win+k"),
            Some(Hotkey { mods: MOD_CONTROL | MOD_ALT | MOD_SHIFT | MOD_WIN, vk: 0x4B })
        );
        assert_eq!(parse_hotkey("Ctrl+\\"), Some(Hotkey { mods: MOD_CONTROL, vk: 0xDC }));
        assert_eq!(parse_hotkey("Ctrl+F12"), Some(Hotkey { mods: MOD_CONTROL, vk: 0x7B }));
        assert_eq!(parse_hotkey("Ctrl+Space"), Some(Hotkey { mods: MOD_CONTROL, vk: 0x20 }));
        assert_eq!(parse_hotkey("X"), None); // needs a modifier
        assert_eq!(parse_hotkey("Ctrl+Shift"), None); // needs a key
        assert_eq!(parse_hotkey("Ctrl+K+L"), None); // one main key only
        assert_eq!(parse_hotkey("Ctrl+Banana"), None);
        assert_eq!(format_hotkey("shift+ctrl+x"), "Ctrl+Shift+X");
        assert_eq!(format_hotkey("win+alt+ctrl+shift+k"), "Ctrl+Alt+Shift+Win+K");
    }

    #[test]
    fn settings_roundtrip() {
        let s = Settings {
            enabled: false,
            direction: Direction::EnToAr,
            last_word: false,
            last_mode: LastMode::Word,
            switch_lang: true,
            hotkey: "Ctrl+Alt+Shift+K".into(),
            digits_hotkey: String::new(),
            snippets_hotkey: "Ctrl+Alt+Shift+E".into(),
        };
        assert_eq!(Settings::parse(&s.serialize()), s);
        assert_eq!(Settings::parse(""), Settings::default());
        // a broken hotkey keeps the default instead of disabling the app
        assert_eq!(Settings::parse("hotkey=nonsense").hotkey, DEFAULT_HOTKEY);
        assert!(Settings::parse("\u{feff}enabled=0").enabled == false);
    }

    #[test]
    fn last_unit_word_and_run() {
        // Arabic word, then two Latin words typed with the wrong layout
        let p = "مرحبا hgsghl ugd;l ";
        let (steps, unit, trail) = last_unit(p, LastMode::Run).unwrap();
        assert_eq!((unit.as_str(), trail.as_str()), ("hgsghl ugd;l", " "));
        assert_eq!(steps, 6);
        let (steps, unit, _) = last_unit(p, LastMode::Word).unwrap();
        assert_eq!((unit.as_str(), steps), ("ugd;l", 13));
        // a single word, caret right after it
        let (steps, unit, trail) = last_unit("hgsghl", LastMode::Run).unwrap();
        assert_eq!((steps, unit.as_str(), trail.as_str()), (0, "hgsghl", ""));
        // numbers do not break a run on their own but never start one
        assert_eq!(last_unit("   ", LastMode::Run), None);
        assert_eq!(last_unit("", LastMode::Run), None);
        // run stops at a token written in the other script
        let (_, unit, _) = last_unit("hello مرحبا اثممخ", LastMode::Run).unwrap();
        assert_eq!(unit, "مرحبا اثممخ");
    }

    #[test]
    fn hotkey_text_roundtrip() {
        for text in ["Ctrl+Shift+X", "Ctrl+Alt+Shift+Win+K", "Ctrl+\\", "Ctrl+F12", "Alt+Space"] {
            let hk = parse_hotkey(text).unwrap();
            assert_eq!(hotkey_text(hk.mods, hk.vk).unwrap(), format_hotkey(text));
        }
        assert_eq!(hotkey_text(0, 0x58), None);
    }

    #[test]
    fn digits_both_ways() {
        assert_eq!(convert_digits("رقم 2026 و 15"), "رقم \u{662}\u{660}\u{662}\u{666} و \u{661}\u{665}");
        assert_eq!(convert_digits("\u{662}\u{660}\u{662}\u{666}"), "2026");
        assert_eq!(convert_digits("\u{6F1}\u{6F2}"), "12");
        assert_eq!(convert_digits("بدون أرقام"), "بدون أرقام");
        // mixed: Western wins, Arabic-Indic ones stay
        assert_eq!(convert_digits("1\u{662}"), "\u{661}\u{662}");
    }

    #[test]
    fn digits_hotkey_setting() {
        assert_eq!(Settings::default().digits_hotkey, "Ctrl+Alt+H");
        assert_eq!(Settings::parse("digits_hotkey=").digits_hotkey, "");
        assert_eq!(Settings::parse("digits_hotkey=ctrl+alt+d").digits_hotkey, "Ctrl+Alt+D");
        assert_eq!(Settings::parse("digits_hotkey=junk").digits_hotkey, "Ctrl+Alt+H");
    }

    #[test]
    fn snippets_roundtrip_and_match() {
        let list = parse_snippets("# c\nemail=me@example.com\nmy address=Line1\\nLine2\nEmail=dup\n");
        assert_eq!(list.len(), 2); // the duplicate trigger is ignored
        assert_eq!(list[1].text, "Line1\nLine2");
        assert_eq!(parse_snippets(&serialize_snippets(&list)), list);
        assert_eq!(lookup_snippet("EMAIL", &list), Some(0));
        // the trigger typed with the Arabic layout still matches
        let wrong = convert("email", Direction::EnToAr);
        assert_eq!(lookup_snippet(&wrong, &list), Some(0));
        assert_eq!(lookup_snippet("emails", &list), None);
    }

    #[test]
    fn snippets_find_at_end_of_line() {
        let list = parse_snippets("email=x\nmy address=y\n");
        let (steps, tail, trail) = find_snippet("send to email ", &list).unwrap();
        assert_eq!((steps, tail.as_str(), trail.as_str()), (8, "email", " "));
        let (steps, tail, _) = find_snippet("hi my address", &list).unwrap();
        assert_eq!((steps, tail.as_str()), (3, "my address"));
        assert_eq!(find_snippet("nothing here", &list), None);
        assert_eq!(find_snippet("", &list), None);
        assert_eq!(expand_placeholders("on {date} at {time}", "2026-11-21", "01:10"), "on 2026-11-21 at 01:10");
    }
}

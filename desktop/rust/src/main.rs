#![cfg_attr(windows, windows_subsystem = "windows")]

mod core;
mod tables;

#[cfg(windows)]
mod win;

#[cfg(windows)]
fn main() {
    win::run();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Baddel for Windows only runs on Windows. Run `cargo test` to check the logic.");
}

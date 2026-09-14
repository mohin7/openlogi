// Hide the console window on Windows release builds. No effect on Linux, but
// keeping it means the crate stays portable.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    openlogi_lib::run()
}

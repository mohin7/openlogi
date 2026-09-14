//! Print the workspace shortcuts resolved from the running desktop.
//! Useful for checking what OpenLogi will actually emit on a given machine.
use button_engine::desktop::{switch_shortcut, workspace_shortcut, Direction, SwitchMode};

fn main() {
    println!("desktop: {}", std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default());
    for (label, move_window) in [("switch", false), ("move window", true)] {
        for d in [Direction::Left, Direction::Right] {
            println!("  {label:<12} {d:?}\t-> {}", workspace_shortcut(d, move_window));
        }
    }
    println!();
    for mode in [
        SwitchMode::Applications,
        SwitchMode::Windows,
        SwitchMode::WindowsOfApp,
        SwitchMode::Overview,
        SwitchMode::AppGrid,
    ] {
        println!("  {mode:?}\t-> {}", switch_shortcut(mode, false));
    }
}

/// App version string used in version output and info screens.
const VERSION: &str = "2.2.0-rs";

/// Builds the long help screen text shown when the user runs `--info`.
/// This gives a friendly overview of controls and purpose.
pub fn info_text() -> String {
    format!(
        r#"
╔═══════════════════════════════════════════════════════════════════════╗
║                                                                       ║
║   Asciiquarium {} - ASCII Art Aquarium Animation                ║
║                                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝

An aquarium/sea animation in ASCII art for your terminal!

CONTROLS:
  Q or q  - Quit the aquarium
  P or p  - Pause/unpause animation
  R or r  - Redraw and respawn entities
  I or i  - Show/hide info screen (press I or ESC to return)
"#,
        VERSION
    )
}

/// Returns a short line-by-line overlay message drawn in the center of the screen.
/// Kept separate from `info_text` so the rendering code stays simple.
pub fn info_lines() -> Vec<String> {
    vec![
        "╔═══════════════════════════════════════════════════════════════════════╗".into(),
        "║                                                                       ║".into(),
        format!("║         Asciiquarium {} - ASCII Art Aquarium Animation          ║", VERSION),
        "║                                                                       ║".into(),
        "╚═══════════════════════════════════════════════════════════════════════╝".into(),
        "".into(),
        "  Q/q quit   P/p pause   R/r reset   I/i info   ESC close info".into(),
        "".into(),
        "  Press I or ESC to return to aquarium...".into(),
    ]
}

/// Builds one compact version line including app version and OS/arch.
/// This is printed for `--version` and `-v`.
pub fn version_string() -> String {
    format!(
        "rustyquarium/{} Rust {}/{}",
        VERSION,
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

use crate::animation::Animation;
use crate::environment::{add_all_seaweed, add_castle, add_environment};
use crate::fish::add_all_fish;
use crate::info::{info_text, version_string};
use crate::special::random_object;

/// Reads command-line flags and decides what to do.
/// It can print info/version text, or start the animation.
/// This function is the main "traffic controller" for CLI behavior.
pub fn run_cli(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut show_info = false;
    let mut show_version = false;
    let mut classic = false;

    for arg in args {
        match arg.as_str() {
            "--info"             => show_info = true,
            "--version" | "-v"  => show_version = true,
            "--classic"         => classic = true,
            other               => return Err(format!("unknown flag: {}", other).into()),
        }
    }

    if show_version {
        println!("{}", version_string());
        return Ok(());
    }
    if show_info {
        print!("{}", info_text());
        return Ok(());
    }

    Animation::new().run(setup_aquarium, classic)
}

/// Builds the scene in a deliberate order.
/// Environment and decor go first, then long-living populations, then one event.
/// This function is reused at startup and after a reset.
fn setup_aquarium(anim: &mut Animation, classic: bool) {
    add_environment(anim);
    add_castle(anim);
    add_all_seaweed(anim);
    add_all_fish(anim, classic);
    random_object(None, anim);
}

use std::time::{Duration, Instant};

use rand::Rng;

use crate::animation::Animation;
use crate::depth::*;
use crate::entity::{CallbackArgs, EntityOptions};

/// Draws repeated water bands near the surface.
/// They are also marked physical so bubbles can collide and pop there.
/// This gives both visuals and simple "surface" collision behavior.
pub fn add_environment(anim: &mut Animation) {
    let segments = [
        "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~",
        "^^^^ ^^^  ^^^   ^^^    ^^^^      ",
        "^^^^      ^^^^     ^^^    ^^     ",
        "^^      ^^^^      ^^^    ^^^^^^  ",
    ];
    let seg_size = segments[0].len();
    let repeat   = anim.width() / seg_size + 1;
    let depth_keys = [DEPTH_WATER_LINE0, DEPTH_WATER_LINE1, DEPTH_WATER_LINE2, DEPTH_WATER_LINE3];

    for (i, seg) in segments.iter().enumerate() {
        let tiled = seg.repeat(repeat);
        anim.new_entity(EntityOptions {
            entity_type:  "waterline".into(),
            shape:        vec![tiled],
            position:     [0, (i + 5) as i32, depth_keys[i]],
            default_color: "CYAN".into(),
            physical:     true,
            ..Default::default()
        });
    }
}

/// Places a decorative castle near the bottom-right.
/// It is static scenery and does not move over time.
pub fn add_castle(anim: &mut Animation) {
    let shape = r#"               T~~
               |
              /^\
             /   \
 _   _   _  /     \  _   _   _
[ ]_[ ]_[ ]/ _   _ \[ ]_[ ]_[ ]
|_=__-_ =_|_[ ]_[ ]_|_=-___-__|
 | _- =  | =_ = _    |= _=   |
 |= -[]  |- = _ =    |_-=_[] |
 | =_    |= - ___    | =_ =  |
 |=  []- |-  /| |\   |=_ =[] |
 |- =_   | =| | | |  |- = -  |
 |_______|__|_|_|_|__|_______|"#;

    let color = r#"                RR
                W
              Wyyw
             y   y
 W   W   W  yWWWWWy  W   W   W
WW WW WW WW W   W WwWW WW WW WW
WWWWWWW WWWWW W W WWWWWWWWWWWWWW
 W W W  W W W W W    W  W   WWW
 W  W   W  W W W     W W W  WWW
 W  W   W  W WWW     W W W  WWW
 W  W   W  W W W W   W  W   WWW
 W  W   W W W W W W  W  W   WWW
 WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW"#;

    let x = anim.width() as i32 - 32;
    let y = anim.height() as i32 - 13;
    anim.new_entity(EntityOptions {
        entity_type:   "castle".into(),
        shape:         vec![shape.into()],
        color:         vec![color.into()],
        position:      [x, y, DEPTH_CASTLE],
        default_color: "DARK_GREY".into(),
        ..Default::default()
    });
}

/// Creates one seaweed with two alternating frames.
/// It does not move position; only the frame index changes to fake swaying.
/// When lifetime ends, the death callback spawns a replacement plant.
pub fn add_seaweed(dead: Option<crate::entity::EntityRef>, anim: &mut Animation) {
    let mut rng = rand::thread_rng();
    let height  = rng.gen_range(3..=6);
    let mut frames = [String::new(), String::new()];
    for i in 1..=height {
        let left = i % 2;
        let right = 1 - left;
        frames[left]  += "(\n";
        frames[right] += " )\n";
    }
    let max_x = (anim.width() as i32 - 2).max(1);
    let x     = rng.gen_range(1..=max_x);
    let y_raw = anim.height() as i32 - height as i32;
    let y     = y_raw.max(9);
    let speed = 0.25 + rng.gen::<f64>() * 0.05;
    let secs  = 8 * 60 + rng.gen_range(0..4 * 60);
    let die_time = Instant::now() + Duration::from_secs(secs);

    let _ = dead; // ignored, matches Go's _ *Entity
    anim.new_entity(EntityOptions {
        entity_type:   "seaweed".into(),
        shape:         frames.into(),
        position:      [x, y, DEPTH_SEAWEED],
        callback_args: Some(CallbackArgs::Move(vec![0.0, 0.0, 0.0, speed])),
        die_time:      Some(die_time),
        death_callback: Some(Box::new(|dead, a| add_seaweed(Some(dead), a))),
        default_color: "GREEN".into(),
        ..Default::default()
    });
}

/// Seeds the initial seaweed population.
/// Count uses a simple width/15 density rule for balanced scenery.
pub fn add_all_seaweed(anim: &mut Animation) {
    let count = anim.width() / 15;
    for _ in 0..count.max(1) {
        add_seaweed(None, anim);
    }
}

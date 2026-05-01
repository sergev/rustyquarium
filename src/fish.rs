use rand::Rng;

use crate::animation::Animation;
use crate::depth::*;
use crate::entity::{CallbackArgs, Entity, EntityOptions, EntityRef};
use crate::special::retract;

// ── Bubble ───────────────────────────────────────────────────────────────────

/// Creates one rising bubble from a fish.
/// `dy = -1` in the callback args means "go up".
/// Bubble starts on the fish mouth side based on horizontal direction.
pub fn add_bubble(fish: EntityRef, anim: &mut Animation) {
    let (bx, by, fz, fw, fh) = {
        let f = fish.borrow();
        let (fx, fy, fz) = f.position();
        let (fw, fh) = f.size();
        let bx = if let CallbackArgs::Move(ref v) = f.callback_args {
            if v.first().copied().unwrap_or(0.0) > 0.0 {
                fx + fw as i32
            } else {
                fx
            }
        } else {
            fx
        };
        (bx, fy + fh as i32 / 2, fz, fw, fh)
    };
    let _ = (fw, fh); // used above
    anim.new_entity(EntityOptions {
        entity_type: "bubble".into(),
        shape: vec![".".into(), "o".into(), "O".into(), "O".into(), "O".into()],
        position: [bx, by, fz - 1],
        callback_args: Some(CallbackArgs::Move(vec![0.0, -1.0, 0.0, 0.1])),
        die_offscreen: true,
        physical: true,
        coll_handler: Some(bubble_collision),
        default_color: "CYAN".into(),
        ..Default::default()
    });
}

/// Removes a bubble when it reaches the water line.
/// This keeps bubbles from floating forever.
fn bubble_collision(bubble: EntityRef, _anim: &mut Animation) {
    let collision: Vec<EntityRef> = bubble.borrow().collision.clone();
    for obj in &collision {
        if obj.borrow().entity_type == "waterline" {
            bubble.borrow_mut().kill();
            return;
        }
    }
}

// ── Fish callbacks ────────────────────────────────────────────────────────────

/// Adds random bubble behavior to fish.
/// After that, it uses normal movement logic.
pub fn fish_callback(fish: EntityRef, anim: &mut Animation) {
    if rand::thread_rng().gen_range(1..=100) > 97 {
        add_bubble(fish.clone(), anim);
    }
    Entity::move_entity(fish, anim);
}

/// Handles dangerous contacts for a fish.
/// Shark teeth can kill small fish; hook contact retracts multiple entities.
/// One collision may change the state of fish, hook, line, and hook point together.
pub fn fish_collision(fish: EntityRef, anim: &mut Animation) {
    let collision: Vec<EntityRef> = fish.borrow().collision.clone();
    for obj in &collision {
        let obj_type = obj.borrow().entity_type.clone();
        if obj_type == "teeth" {
            let (_, fh) = fish.borrow().size();
            if fh <= 5 {
                let (ox, oy, oz) = obj.borrow().position();
                add_splat(anim, ox, oy, oz);
                fish.borrow_mut().kill();
            }
            return;
        }
        if obj_type == "hook_point" {
            retract(obj.clone(), anim);
            retract(fish.clone(), anim);
            for h in anim.get_entities_by_type("fishhook") {
                retract(h, anim);
            }
            for l in anim.get_entities_by_type("fishline") {
                retract(l, anim);
            }
            return;
        }
    }
}

// ── Splat ────────────────────────────────────────────────────────────────────

/// Creates a short visual burst when a fish gets eaten.
/// `die_frame` means "remove after enough frame steps," not wall-clock seconds.
pub fn add_splat(anim: &mut Animation, x: i32, y: i32, z: i32) {
    let frames = vec![
        "\n\n   .\n  ***\n   '\n\n".into(),
        "\n\n .,*;`\n '*,**\n *'~'\n\n".into(),
        "\n  , ,\n \" ,\"'\n *\" *'\"\n  \" ; .\n\n".into(),
        "* ' , ' `\n' ` * . '\n ' `' \",'\n* ' \" * .\n\" * ', '".into(),
    ];
    anim.new_entity(EntityOptions {
        shape: frames,
        position: [x - 4, y - 2, z - 2],
        default_color: "RED".into(),
        callback_args: Some(CallbackArgs::Move(vec![0.0, 0.0, 0.0, 0.25])),
        auto_trans: true,
        die_frame: 15,
        ..Default::default()
    });
}

// ── Fish designs ──────────────────────────────────────────────────────────────

/// Stores left/right sprite variants and color masks for one fish design.
/// We pick one direction pair based on travel direction.
struct FishDesign {
    shape: [&'static str; 2],
    color: [&'static str; 2],
}

static OLD_FISH: &[FishDesign] = &[
    FishDesign {
        shape: [
            "       \\\n     ...\\..,\n\\  /'       \\\n >=     (  ' >\n/  \\      / /\n    `\"'\"'/''",
            "      /\n  ,../...\n /       '\\  /\n< '  )     =<\n \\ \\      /  \\\n  `'\"'\"'",
        ],
        color: [
            "       2\n     1112111\n6  11       1\n 66     7  4 5\n6  1      3 1\n    11111311",
            "      2\n  1112111\n 1       11  6\n5 4  7     66\n 1 3      1  6\n  11311111",
        ],
    },
    FishDesign {
        shape: [
            "    \\\n\\ /--\\\n>=  (o>\n/ \\__/\n    /",
            "  /\n /--\\ /\n<o)  =<\n \\__/ \\\n  \\",
        ],
        color: [
            "    2\n6 1111\n66  745\n6 1111\n    3",
            "  2\n 1111 6\n547  66\n 1111 6\n  3",
        ],
    },
    FishDesign {
        shape: [
            "       \\:.\n\\;,   ,;\\\\\\\\,,\n  \\\\\\;;:::::::o\n  ///;;::::::::<\n /;` ``/////``",
            "      .:/\n   ,,///;,   ,;/\n o:::::::;;///\n>::::::::;;\\\\\\\\\n  ''\\\\\\\\\\\\\\\\'' ';\\",
        ],
        color: [
            "       222\n666   1122211\n  6661111111114\n  66611111111115\n 666 113333311",
            "      222\n   1122211   666\n 4111111111666\n51111111111666\n  113333311 666",
        ],
    },
    FishDesign {
        shape: [
            "  __\n><_'>\n   '",
            " __\n<'_><\n `",
        ],
        color: [
            "  11\n61145\n   3",
            " 11\n54116\n 3",
        ],
    },
    FishDesign {
        shape: [
            "   ..\\\\\n>='   ('>\n  '''/''",
            "  ,..\n<')   `=<\n ``\\```",
        ],
        color: [
            "   1121\n661   745\n  111311",
            "  1211\n547   166\n 113111",
        ],
    },
    FishDesign {
        shape: [
            "   \\\n  / \\\n>=_('>\n  \\_/\n   /",
            "  /\n / \\\n<')_=<\n \\_/\n  \\",
        ],
        color: [
            "   2\n  1 1\n661745\n  111\n   3",
            "  2\n 1 1\n547166\n 111\n  3",
        ],
    },
    FishDesign {
        shape: [
            "  ,\\\n>=('>\n  '/",
            " /,\n<')=<\n \\`",
        ],
        color: [
            "  12\n66745\n  13",
            " 21\n54766\n 31",
        ],
    },
    FishDesign {
        shape: [
            "  __\n\\/ o\\\n/\\__/",
            " __\n/o \\/\n\\__/\\",
        ],
        color: [
            "  11\n61 41\n61111",
            " 11\n14 16\n11116",
        ],
    },
];

static NEW_FISH: &[FishDesign] = &[
    FishDesign {
        shape: [
            "   \\\n  / \\\n>=_('>\n  \\_/\n   /",
            "  /\n / \\\n<')_=<\n \\_/\n  \\",
        ],
        color: [
            "   1\n  1 1\n663745\n  111\n   3",
            "  2\n 111\n547366\n 111\n  3",
        ],
    },
    FishDesign {
        shape: [
            "     ,\n     }\\\\\n\\  .'  `\\\n}}<   ( 6>\n/  `,  .'\n     }/\n     '",
            "    ,\n   /{\n /'  `.  /\n<6 )   >{{\n `.  ,'  \\\n   {\\\n    `",
        ],
        color: [
            "     2\n     22\n6  11  11\n661   7 45\n6  11  11\n     33\n     3",
            "    2\n   22\n 11  11  6\n54 7   166\n 11  11  6\n   33\n    3",
        ],
    },
    FishDesign {
        shape: [
            "            \\'`.\n             )  \\\n(`.      _.-`' ' '`-.\n \\ `.  .`        (o) \\_\n  >  ><     (((       (\n / .`  ._      /_|  /'\n(.`       `-. _  _.-`\n            /__/'",
            "       .'`/\n      /  (\n  .-'` ` `'-._      .')\n_/ (o)        '.  .' /\n)       )))     ><  <\n`\\  |_\\      _.'  '. \\\n  '-._  _ .-'       '.)\n      `\\__\\",
        ],
        color: [
            "            1111\n             1  1\n111      11111 1 1111\n 1 11  11        141 11\n  1  11     777       5\n 1 11  111      333  11\n111       111 1  1111\n            11111",
            "       1111\n      1  1\n  1111 1 11111      111\n11 141        11  11 1\n5       777     11  1\n11  333      111  11 1\n  1111  1 111       111\n      11111",
        ],
    },
    FishDesign {
        shape: [
            "       ,--,_\n__    _\\.---'-.\n\\ '.-\"     // o\\\n/_.'-._    \\\\  /\n       `\"--(/\"`",
            "    _,--,\n .-'---./_    __\n/o \\\\     \"-.' /\n\\  //    _.-'._\\\n `\"\\)--\"`",
        ],
        color: [
            "       22222\n66    121111211\n6 6111     77 41\n6661111    77  1\n       11113311",
            "    22222\n 112111121    66\n14 77     1116 6\n1  77    1111666\n 11331111",
        ],
    },
];

/// Replaces numeric color placeholders in sprite masks.
/// Digits 1–9 are templates that become random color marker letters.
/// This keeps fish colors varied without changing the shape art.
fn rand_color(mask: &str) -> String {
    let palette = ["c", "C", "r", "R", "y", "Y", "b", "B", "g", "G", "m", "M"];
    let mut rng = rand::thread_rng();
    let mut out = mask.to_string();
    for i in 1..=9u8 {
        let digit = (b'0' + i) as char;
        let replacement = palette[rng.gen_range(0..palette.len())];
        out = out.replace(digit, replacement);
    }
    out
}

/// Spawns one fish using mode rules and a random design.
/// Direction chooses speed sign and which side of the screen the fish starts from.
/// The death callback respawns another fish, keeping the population stable.
pub fn add_fish(dead: Option<EntityRef>, anim: &mut Animation, classic: bool) {
    let mut rng = rand::thread_rng();
    let (shape_pair, color_pair) = if classic || rng.gen_range(1..=12) <= 8 {
        let d = &OLD_FISH[rng.gen_range(0..OLD_FISH.len())];
        (d.shape, d.color)
    } else {
        let d = &NEW_FISH[rng.gen_range(0..NEW_FISH.len())];
        (d.shape, d.color)
    };

    let direction: usize = rng.gen_range(0..2);
    let mut speed = 0.25 + rng.gen::<f64>() * 1.75;
    if direction == 1 {
        speed = -speed;
    }
    let depth = DEPTH_FISH_START + rng.gen_range(0..=(DEPTH_FISH_END - DEPTH_FISH_START));

    let shape = vec![shape_pair[direction].to_string()];
    let color = vec![rand_color(color_pair[direction])];

    let _ = dead;
    let fish = Entity::new(EntityOptions {
        entity_type: "fish".into(),
        shape,
        color,
        auto_trans: true,
        position: [0, 0, depth],
        callback: Some(fish_callback),
        callback_args: Some(CallbackArgs::Move(vec![speed, 0.0, 0.0])),
        die_offscreen: true,
        physical: true,
        coll_handler: Some(fish_collision),
        death_callback: Some(Box::new(move |_, a| add_fish(None, a, classic))),
        ..Default::default()
    });

    let water_bottom = 9;
    let screen_bottom = anim.height() as i32 - 1;
    let (fw, fh) = fish.borrow().size();
    let available = screen_bottom - water_bottom - fh as i32;
    fish.borrow_mut().y = if available > 0 {
        (water_bottom + rng.gen_range(0..=available)) as f64
    } else {
        water_bottom as f64
    };
    fish.borrow_mut().x = if direction == 0 {
        -(fw as f64)
    } else {
        anim.width() as f64
    };
    anim.add_entity(fish);
}

/// Creates the initial fish count from screen area.
/// The /350 constant is a simple density tuning value.
pub fn add_all_fish(anim: &mut Animation, classic: bool) {
    let screen_size = (anim.height().saturating_sub(9)) * anim.width();
    let count = (screen_size / 350).max(1);
    for _ in 0..count {
        add_fish(None, anim, classic);
    }
}

#[cfg(test)]
mod tests {
    use super::add_fish;
    use super::rand_color;
    use super::{NEW_FISH, OLD_FISH};
    use std::collections::HashSet;
    use std::rc::Rc;

    fn make_anim() -> crate::animation::Animation {
        let mut a = crate::animation::Animation::new();
        a.width = 120;
        a.height = 40;
        a
    }

    // Mirrors Go's TestRandColor.
    #[test]
    fn test_rand_color_replaces_digits_with_valid_color_chars() {
        let mask = rand_color("123456789");
        assert_eq!(
            mask.chars().count(),
            9,
            "length should be preserved after digit replacement"
        );
        for ch in mask.chars() {
            match ch {
                'c' | 'C' | 'r' | 'R' | 'y' | 'Y' | 'b' | 'B' | 'g' | 'G' | 'm' | 'M' => {}
                other => panic!("unexpected color char {:?} in rand_color output", other),
            }
        }
    }

    #[test]
    fn test_rand_color_leaves_non_digits_unchanged() {
        let mask = rand_color("abc!@#");
        assert_eq!(
            mask, "abc!@#",
            "non-digit chars must pass through unchanged"
        );
    }

    // Mirrors Go's TestFishDesignCatalogParityCounts.
    #[test]
    fn test_fish_design_catalog_counts() {
        assert_eq!(OLD_FISH.len(), 8, "expected 8 old fish designs");
        assert_eq!(NEW_FISH.len(), 4, "expected 4 new fish designs");
    }

    // Mirrors Go's TestAddFishAlwaysHasValidShape.
    // Runs 200 iterations per mode to cover all 12 designs × 2 directions.
    #[test]
    fn test_add_fish_always_has_valid_shape() {
        for classic in [false, true] {
            let mut a = make_anim();
            for i in 0..200usize {
                let before: HashSet<*const _> = a.entities.iter().map(|e| Rc::as_ptr(e)).collect();
                add_fish(None, &mut a, classic);
                let fish = a
                    .entities
                    .iter()
                    .find(|e| !before.contains(&Rc::as_ptr(e)))
                    .cloned()
                    .unwrap_or_else(|| {
                        panic!(
                            "classic={} iter {}: add_fish added no new entity",
                            classic, i
                        )
                    });
                let b = fish.borrow();
                assert_eq!(
                    b.entity_type, "fish",
                    "classic={} iter {}: entity_type should be \"fish\"",
                    classic, i
                );
                assert!(
                    !b.current_shape().is_empty(),
                    "classic={} iter {}: fish entity has empty shape",
                    classic,
                    i
                );
            }
        }
    }

    // Mirrors Go's TestNewEntityShapeNotCorruptedByColorArg.
    // Guards against GC-style use-after-free bugs by verifying shapes survive after rand_color.
    #[test]
    fn test_fish_shape_not_corrupted_by_color_arg() {
        for i in 0..500usize {
            let design = &OLD_FISH[i % OLD_FISH.len()];
            let direction = i % 2;
            let shape_val = design.shape[direction].to_string();
            let color_val = rand_color(design.color[direction]);
            let e = crate::entity::Entity::new(crate::entity::EntityOptions {
                entity_type: "fish".into(),
                shape: vec![shape_val.clone()],
                color: vec![color_val],
                ..Default::default()
            });
            assert_eq!(
                e.borrow().current_shape(),
                shape_val,
                "iter {}: fish shape corrupted after rand_color allocation",
                i
            );
        }
    }
}

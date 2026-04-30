use rand::Rng;

use crate::animation::Animation;
use crate::depth::*;
use crate::entity::{CallbackArgs, EntityOptions, EntityRef};

// ── Shark ─────────────────────────────────────────────────────────────────────

/// Creates two linked entities: the shark art and the teeth hitbox.
/// Keeping teeth separate makes collision checks simple and precise.
/// The shark death callback removes teeth and starts the next event.
pub fn add_shark(_dead: Option<EntityRef>, anim: &mut Animation) {
    let shapes = [
        "                              __\n                             ( `\\\n  ,                          )   `\\\n;' `.                       (     `\\__\n ;   `.             __..---''          `~~~~-._\n  `.   `.____...--''                       (b  `--._\n    >                     _.-'      .((      ._     )\n  .`.-`--...__         .-'     -.___.....-(|/|/|/|/'\n ;.'         `. ...----`.___.',,,_______......---'\n '           '-'",
        "                     __\n                    /' )\n                  /'   (                          ,\n              __/'     )                       .' `;\n      _.-~~~~'          ``---..__             .'   ;\n _.--'  b)                       ``--...____.'   .'\n(     _.      )).      `-._                     <\n `\\|\\|\\|\\|)-.....___.-     `-.         __...--'-.'.\n   `---......_______,,,`.___.'----... .'         `.;\n                                     `-`           `",
    ];
    let colors = [
        "\n\n\n\n\n                                           cR\n \n                                          cWWWWWWWW\n\n\n",
        "\n\n\n\n        Rc\n\n  WWWWWWWWc\n\n\n\n",
    ];
    let mut rng   = rand::thread_rng();
    let direction = rng.gen_range(0..2);
    let mut y     = 9i32;
    let mut teeth_y;
    let speed;
    let x;
    let teeth_x;

    if anim.height() as i32 > 19 {
        y += rng.gen_range(0..=(anim.height() as i32 - 19).max(1));
    }
    teeth_y = y + 7;

    if direction == 0 {
        speed   = 2.0f64;
        x       = -53i32;
        teeth_x = -9i32;
    } else {
        speed   = -2.0f64;
        x       = anim.width() as i32 - 2;
        teeth_x = x + 9;
    }
    let _ = teeth_y; // used below
    teeth_y = y + 7;

    anim.new_entity(EntityOptions {
        entity_type:   "teeth".into(),
        shape:         vec!["*".into()],
        position:      [teeth_x, teeth_y, DEPTH_SHARK + 1],
        callback_args: Some(CallbackArgs::Move(vec![speed, 0.0, 0.0])),
        physical:      true,
        ..Default::default()
    });
    anim.new_entity(EntityOptions {
        entity_type:    "shark".into(),
        shape:          vec![shapes[direction].into()],
        color:          vec![colors[direction].into()],
        auto_trans:     true,
        position:       [x, y, DEPTH_SHARK],
        default_color:  "CYAN".into(),
        callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0])),
        die_offscreen:  true,
        death_callback: Some(Box::new(|_, a| shark_death(a))),
        ..Default::default()
    });
}

/// Removes teeth when the shark exits.
/// It then spawns another random special object.
fn shark_death(anim: &mut Animation) {
    for t in anim.get_entities_by_type("teeth") {
        anim.del_entity(&t);
    }
    random_object(None, anim);
}

// ── Ship ──────────────────────────────────────────────────────────────────────

/// Spawns a boat moving on the surface layer.
/// It includes a color mask so sails and hull have detail.
pub fn add_ship(_dead: Option<EntityRef>, anim: &mut Animation) {
    let shapes = [
        "     |    |    |\n    )_)  )_)  )_)\n   )___))___))___)\\  \n  )____)____)_____)\\\\  \n_____|____|____|____\\\\\\__\n\\                   /",
        "         |    |    |\n        (_(  (_(  (_(\n      /(___((___((___(\n    //(_____(____(____(  \n__///____|____|____|_____\n    \\                   /",
    ];
    let colors = [
        "     y    y    y\n\n                  w\n                   ww\nyyyyyyyyyyyyyyyyyyyywwwyy\ny                   y",
        "         y    y    y\n\n      w\n    ww\nyywwwyyyyyyyyyyyyyyyyyyyy\n    y                   y",
    ];
    let mut rng = rand::thread_rng();
    let dir = rng.gen_range(0..2);
    let speed;
    let x;
    if dir == 0 { speed = 1.0f64;  x = -24i32; }
    else        { speed = -1.0f64; x = anim.width() as i32 - 2; }
    anim.new_entity(EntityOptions {
        shape:          vec![shapes[dir].into()],
        color:          vec![colors[dir].into()],
        auto_trans:     true,
        position:       [x, 0, DEPTH_WATER_GAP1],
        default_color:  "WHITE".into(),
        callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0, 0.0])),
        die_offscreen:  true,
        death_callback: Some(Box::new(|_, a| random_object(None, a))),
        ..Default::default()
    });
}

// ── Whale ─────────────────────────────────────────────────────────────────────

/// Builds animation frames in code instead of hardcoding all frames.
/// Starts with idle body frames, then appends aligned spout variations.
/// The same color masks are reused while shape frames change over time.
pub fn add_whale(_dead: Option<EntityRef>, anim: &mut Animation) {
    let shapes = [
        "        .-----:\n      .'       `.\n,    /       (o) \\\n\\`._/          ,__)",
        "    :-----.\n  .'       `.\n / (o)       \\    ,\n(__,          \\_.'/",
    ];
    let colors = [
        "             C C\n           CCCCCCC\n           C  C  C\n        BBBBBBB\n      BB       BB\nB    B       BWB B\nBBBBB          BBBB",
        "   C C\n CCCCCCC\n C  C  C\n    BBBBBBB\n  BB       BB\n B BWB       B    B\nBBBB          BBBBB",
    ];
    let spouts = [
        "\n\n\n   :",
        "\n\n   :\n   :",
        "\n  . .\n  -:-\n   :",
        "\n  . .\n .-:-.\n   :",
        "\n  . .\n'.-:-.`\n'  :  '",
        "\n\n .- -.\n;  :  ;",
        "\n\n\n;     ;",
    ];

    let mut rng = rand::thread_rng();
    let dir = rng.gen_range(0..2);
    let speed;
    let x;
    let spout_align;
    if dir == 0 { speed = 0.5f64;  x = -18i32; spout_align = 11; }
    else        { speed = -0.5f64; x = anim.width() as i32 - 2; spout_align = 1; }

    let mut anim_shapes: Vec<String> = Vec::new();
    let mut anim_colors: Vec<String> = Vec::new();
    for _ in 0..5 {
        anim_shapes.push(format!("\n\n\n{}", shapes[dir]));
        anim_colors.push(colors[dir].into());
    }
    for spout in &spouts {
        let lines: Vec<&str> = spout.split('\n').collect();
        let pad = " ".repeat(spout_align);
        let sep = format!("\n{}", pad);
        let aligned = lines.join(&sep);
        anim_shapes.push(format!("{}\n{}", aligned, shapes[dir]));
        anim_colors.push(colors[dir].into());
    }

    anim.new_entity(EntityOptions {
        shape:          anim_shapes,
        color:          anim_colors,
        auto_trans:     true,
        position:       [x, 0, DEPTH_WATER_GAP2],
        default_color:  "WHITE".into(),
        callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0, 1.0])),
        die_offscreen:  true,
        death_callback: Some(Box::new(|_, a| random_object(None, a))),
        ..Default::default()
    });
}

// ── Monster ───────────────────────────────────────────────────────────────────

/// Spawns a sea monster crossing the screen.
/// It picks between the newer and classic animated monster designs.
pub fn add_monster(dead: Option<EntityRef>, anim: &mut Animation) {
    let mut rng = rand::thread_rng();
    if rng.gen_range(0..2) == 0 {
        add_new_monster(dead, anim);
    } else {
        add_old_monster(dead, anim);
    }
}

/// Creates the larger modern monster variant.
/// It uses two animation frames and keeps the eye highlight mask.
fn add_new_monster(_dead: Option<EntityRef>, anim: &mut Animation) {
    let shapes: [[&str; 2]; 2] = [
        [
            "\n         _   _                   _   _       _a_a\n       _{.`=`.}_     _   _     _{.`=`.}_    {/ ''\\_\n _    {.'  _  '.}   {.`'`.}   {.'  _  '.}  {|  ._oo)\n{ \\  {/  .'~'.  \\}  {/ .-. \\}  {/  .'~'.  \\} {/  |",
            "\n                      _   _                    _a_a\n  _      _   _     _{.`=`.}_     _   _      {/ ''\\_\n { \\    {.`'`.}   {.'  _  '.}   {.`'`.}    {|  ._oo)\n  \\ \\  {/ .-. \\}  {/  .'~'.  \\}  {/ .-. \\}   {/  |",
        ],
        [
            "\n   a_a_       _   _                   _   _\n _/'' \\}    _{.`=`.}_     _   _     _{.`=`.}_\n(oo_.  |}  {.'  _  '.}   {.`'`.}   {.'  _  '.}    _\n    |  \\} {/  .'~'.  \\}  {/ .-. \\}  {/  .'~'.  \\}  / }",
            "\n   a_a_                    _   _\n _/'' \\}      _   _     _{.`=`.}_     _   _      _\n(oo_.  |}    {.`'`.}   {.'  _  '.}   {.`'`.}    / }\n    |  \\}   {/ .-. \\}  {/  .'~'.  \\}  {/ .-. \\}  / /",
        ],
    ];
    let colors = [
        "\n                                                W W\n\n\n\n",
        "\n   W W\n\n\n\n",
    ];
    let mut rng = rand::thread_rng();
    let dir = rng.gen_range(0..2);
    let speed;
    let x;
    if dir == 0 { speed = 2.0f64;  x = -54i32; }
    else        { speed = -2.0f64; x = anim.width() as i32 - 2; }

    anim.new_entity(EntityOptions {
        shape:          shapes[dir].iter().map(|s| s.to_string()).collect(),
        color:          vec![colors[dir].into(), colors[dir].into()],
        auto_trans:     true,
        position:       [x, 2, DEPTH_WATER_GAP2],
        callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0, 0.25])),
        death_callback: Some(Box::new(|_, a| random_object(None, a))),
        die_offscreen:  true,
        default_color:  "GREEN".into(),
        ..Default::default()
    });
}

/// Creates the classic sea monster variant.
/// This one uses four animation frames for the body wake motion.
fn add_old_monster(_dead: Option<EntityRef>, anim: &mut Animation) {
    let shapes: [[&str; 4]; 2] = [
        [
            "\n                                                          ____\n            __                                          /   o  \\\n          /    \\        _                     _       /     ____ >\n  _      |  __  |     /   \\        _        /   \\   |     |\n | \\     |  ||  |    |     |     /   \\    |     |  |     |",
            "\n                                                          ____\n                                             __         /   o  \\\n             _                     _       /    \\     /     ____ >\n   _       /   \\        _        /   \\   |  __  |   |     |\n  | \\     |     |     /   \\    |     |  |  ||  |   |     |",
            "\n                                                          ____\n                                  __                  /   o  \\\n _                      _       /    \\        _     /     ____ >\n| \\          _        /   \\   |  __  |     /   \\  |     |\n \\ \\       /   \\    |     |  |  ||  |    |     | |     |",
            "\n                                                          ____\n                       __                             /   o  \\\n  _          _       /    \\        _                /     ____ >\n | \\       /   \\   |  __  |     /   \\        _    |     |\n  \\ \\     |     |  |  ||  |    |     |     /   \\  |     |",
        ],
        [
            "\n    ____\n  /  o   \\                                          __\n< ____     \\       _                     _        /    \\\n      |     |   /   \\        _        /   \\     |  __  |      _\n      |     |  |     |     /   \\    |     |    |  ||  |     / |",
            "\n    ____\n  /  o   \\         __\n< ____     \\     /    \\       _                     _\n      |     |   |  __  |    /   \\        _        /   \\       _\n      |     |   |  ||  |   |     |     /   \\     |     |     / |",
            "\n    ____\n  /  o   \\                  __\n< ____     \\     _        /    \\       _                      _\n      |     |  /   \\     |  __  |   /   \\        _          / |\n      |     | |     |    |  ||  |  |     |    /   \\       / /",
            "\n    ____\n  /  o   \\                             __\n< ____     \\                _        /    \\       _          _\n      |     |    _        /   \\     |  __  |   /   \\       / |\n      |     |  /   \\    |     |    |  ||  |  |     |     / /",
        ],
    ];
    let colors = [
        "\n\n                                                            W\n\n\n",
        "\n\n     W\n\n\n",
    ];
    let mut rng = rand::thread_rng();
    let dir = rng.gen_range(0..2);
    let speed;
    let x;
    if dir == 0 { speed = 2.0f64;  x = -64i32; }
    else        { speed = -2.0f64; x = anim.width() as i32 - 2; }

    let color_frames = vec![
        colors[dir].to_string(),
        colors[dir].to_string(),
        colors[dir].to_string(),
        colors[dir].to_string(),
    ];
    anim.new_entity(EntityOptions {
        shape:          shapes[dir].iter().map(|s| s.to_string()).collect(),
        color:          color_frames,
        auto_trans:     true,
        position:       [x, 2, DEPTH_WATER_GAP2],
        callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0, 0.25])),
        death_callback: Some(Box::new(|_, a| random_object(None, a))),
        die_offscreen:  true,
        default_color:  "GREEN".into(),
        ..Default::default()
    });
}

// ── Big fish ──────────────────────────────────────────────────────────────────

/// Creates a larger fast fish variant.
/// Keeps upstream weighting: design 2 appears 2/3 of the time.
pub fn add_big_fish(dead: Option<EntityRef>, anim: &mut Animation) {
    let mut rng = rand::thread_rng();
    if rng.gen_range(0..3) > 0 {
        add_big_fish2(dead, anim);
    } else {
        add_big_fish1(dead, anim);
    }
}

/// Replaces numeric color placeholders in sprite masks.
/// Digits 1–9 become random color marker letters so big fish colors vary each run.
fn rand_color(mask: &str) -> String {
    let palette = ["c", "C", "r", "R", "y", "Y", "b", "B", "g", "G", "m", "M"];
    let mut rng = rand::thread_rng();
    let mut out = mask.to_string();
    for i in 1..=9u8 {
        let digit = (b'0' + i) as char;
        let repl  = palette[rng.gen_range(0..palette.len())];
        out = out.replace(digit, repl);
    }
    out
}

/// First big fish design: a large elongated fish with colored body detail.
fn add_big_fish1(_dead: Option<EntityRef>, anim: &mut Animation) {
    let shapes = [
        " ______\n`\"\".  `````-----.....__\n     `.  .      .       `-.\n       :     .     .       `.\n ,     :   .    .          _ :\n: `.   :                  (@) `._\n `. `..'     .     =`-.       .__)\n   ;     .        =  ~  :     .-\"\n .' .'`.   .    .  =.-'  `._ .'\n: .'   :               .   .'\n '   .'  .    .     .   .-'\n   .'____....----''.'=.'\n   \"\"             .'.'\n               ''\"'`",
        "                           ______\n          __.....-----'''''  .-\"\"'\n       .-'       .      .  .\n     .'       .     .     :\n    : _          .    .   :     ,\n _.' (@)                  :   .' :\n(__.       .-'=     .     `..' .'\n \"-.     :  ~  =        .     ;\n   `. _.'  `-.=  .    .   .'`. `.\n     `.   .               :   `. :\n       `-.   .     .    .  `.   `\n          `.=`.``----....____`.\n            `.`.             \"\"\n              '`\"``",
    ];
    let colors = [
        " 111111\n11111  11111111111111111\n     11  2      2       111\n       1     2     2       11\n 1     1   2    2          1 1\n1 11   1                  1W1 111\n 11 1111     2     1111       1111\n   1     2        1  1  1     111\n 11 1111   2    2  1111  111 11\n1 11   1               2   11\n 1   11  2    2     2   111\n   111111111111111111111\n   11             1111\n               11111",
        "                           111111\n          11111111111111111  11111\n       111       2      2  11\n     11       2     2     1\n    1 1          2    2   1     1\n 111 1W1                  1   11 1\n1111       1111     2     1111 11\n 111     1  1  1        2     1\n   11 111  1111  2    2   1111 11\n     11   2               1   11 1\n       111   2     2    2  11   1\n          111111111111111111111\n            1111             11\n              11111",
    ];
    let mut rng = rand::thread_rng();
    let dir = rng.gen_range(0..2);
    let speed;
    let x;
    if dir == 0 { speed = 3.0f64;  x = -34i32; }
    else        { speed = -3.0f64; x = anim.width() as i32 - 1; }
    let max_h = 9;
    let min_h = anim.height() as i32 - 15;
    let y = if min_h > max_h { max_h + rng.gen_range(0..=(min_h - max_h)) } else { max_h };
    anim.new_entity(EntityOptions {
        shape:          vec![shapes[dir].into()],
        color:          vec![rand_color(colors[dir])],
        auto_trans:     true,
        position:       [x, y, DEPTH_SHARK],
        callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0])),
        death_callback: Some(Box::new(|_, a| random_object(None, a))),
        die_offscreen:  true,
        default_color:  "YELLOW".into(),
        ..Default::default()
    });
}

/// Second big fish design: a scaled fish with a diagonal stripe pattern.
fn add_big_fish2(_dead: Option<EntityRef>, anim: &mut Animation) {
    let shapes = [
        "                _ _ _\n             .='\\ \\ \\`\"=,\n           .'\\ \\ \\ \\ \\ \\ \\\n\\'=._     / \\ \\ \\_\\_\\_\\_\\_\\\n\\'=._'.  /\\ \\,-\"`- _ - _ - '-.\n  \\`=._\\|'.\\/- _ - _ - _ - _- \\\n  ;\"= ._\\=./_ -_ -_ {`\"=_    @ \\\n   ;=\"_-_=- _ -  _ - {\"=_\"-     \\\n   ;_=_--_.,          {_.='   .-/\n  ;.=\"` / ';\\        _.     _.-`\n  /_.='/ \\/ /;._ _ _{.-;`/\"\n/._=_.'   '/ / / / /{.= /\n/.='       `'./_/_.=`{_/",
        "            _ _ _\n        ,=\"`/ / /'=. \n       / / / / / / /'.\n      /_/_/_/_/_/ / / \\     _.='/\n   .-' - _ - _ -`\"-,/ /\\  .'_.='/\n  / -_ - _ - _ - _ -\\/.'|/_.=`/\n / @    _=\"`} _- _- _\\.=/_. =\";\n/     -\"_=\"}  - _  - _ -=_-_\"=;\n\\-.   '=._}          ,._--_=_; \n `-._     ._        /;' \\ `\"=.;\n     `\"\\`;-.}_ _ _.;\\ \\/ \\'=._\\\n        \\ =.}\\ \\ \\ \\ \\'   '._=_.\\\n         \\_}`=._\\_\\.'`       '=.\\",
    ];
    let colors = [
        "                1 1 1\n             1111 1 11111\n           111 1 1 1 1 1 1\n11111     1 1 1 11111111111\n1111111  11 111112 2 2 2 2 111\n  111111111112 2 2 2 2 2 2 22 1\n  111 1111 12 22 22 11111    W 1\n   11111112 2 2  2 2 111111     1\n   111111111          11111   111\n  11111 11111        11     1111\n  111111 11 1111 1 111111111\n1111111   11 1 1 1 1111 1\n1111       1111111111111",
        "            1 1 1\n        11111 1 1111\n       1 1 1 1 1 1 111\n      11111111111 1 1 1     11111\n   111 2 2 2 2 211111 11  1111111\n  1 22 2 2 2 2 2 2 211111111111\n 1 W    11111 22 22 2111111 111\n1     111111 2 2  2 2 21111111\n111   11111          111111111\n 1111     11        111 1 11111\n     111111111 1 1111 11 111111\n        1 1111 1 1 1 11   1111111\n         1111111111111       1111",
    ];
    let mut rng = rand::thread_rng();
    let dir = rng.gen_range(0..2);
    let speed;
    let x;
    if dir == 0 { speed = 2.5f64;  x = -33i32; }
    else        { speed = -2.5f64; x = anim.width() as i32 - 1; }
    let max_h = 9;
    let min_h = anim.height() as i32 - 14;
    let y = if min_h > max_h { max_h + rng.gen_range(0..=(min_h - max_h)) } else { max_h };
    anim.new_entity(EntityOptions {
        shape:          vec![shapes[dir].into()],
        color:          vec![rand_color(colors[dir])],
        auto_trans:     true,
        position:       [x, y, DEPTH_SHARK],
        callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0])),
        death_callback: Some(Box::new(|_, a| random_object(None, a))),
        die_offscreen:  true,
        default_color:  "YELLOW".into(),
        ..Default::default()
    });
}

// ── Fishhook ──────────────────────────────────────────────────────────────────

/// Creates a 3-part system: line, visible hook, and catch point.
/// All parts share the same callback mode so they move together.
/// The hook point is the physical part that a fish collides with.
pub fn add_fishhook(_dead: Option<EntityRef>, anim: &mut Animation) {
    let mut rng = rand::thread_rng();
    let x       = 10 + rng.gen_range(0..(anim.width() as i32 - 30).max(1));
    let y_start = -20i32;
    let y_line  = y_start - 1;
    let line_str: String = "|\n".repeat(50);

    anim.new_entity(EntityOptions {
        entity_type:    "fishline".into(),
        shape:          vec![line_str],
        position:       [x + 7, y_line, DEPTH_WATER_LINE1],
        auto_trans:     true,
        callback:       Some(fishhook_callback),
        callback_args:  Some(CallbackArgs::State({
            let mut m = std::collections::HashMap::new();
            m.insert("mode".into(), "lowering".into());
            m
        })),
        ..Default::default()
    });
    anim.new_entity(EntityOptions {
        entity_type:    "fishhook".into(),
        shape:          vec!["       o\n      ||\n      ||\n/ \\   ||\n  \\__//\n  `--'".into()],
        position:       [x, y_start, DEPTH_WATER_LINE1],
        auto_trans:     true,
        // Spawn above the viewport, so offscreen death must stay disabled while lowering.
        die_offscreen:  false,
        default_color:  "GREEN".into(),
        callback:       Some(fishhook_callback),
        callback_args:  Some(CallbackArgs::State({
            let mut m = std::collections::HashMap::new();
            m.insert("mode".into(), "lowering".into());
            m
        })),
        death_callback: Some(Box::new(|_, a| {
            group_death(a, &["hook_point", "fishline"]);
        })),
        ..Default::default()
    });
    anim.new_entity(EntityOptions {
        entity_type:    "hook_point".into(),
        shape:          vec![".\n \n\\\n ".into()],
        position:       [x + 1, y_start + 2, DEPTH_SHARK + 1],
        physical:       true,
        default_color:  "GREEN".into(),
        callback:       Some(fishhook_callback),
        callback_args:  Some(CallbackArgs::State({
            let mut m = std::collections::HashMap::new();
            m.insert("mode".into(), "lowering".into());
            m
        })),
        ..Default::default()
    });
}

/// Acts like a tiny state machine.
/// "lowering" moves down to max depth; "hooked" reels upward to the top clamp.
/// This callback uses `State` map args, unlike most entities' `Move` args.
fn fishhook_callback(entity: EntityRef, anim: &mut Animation) {
    let mode = {
        let e = entity.borrow();
        if let CallbackArgs::State(ref m) = e.callback_args {
            m.get("mode").cloned().unwrap_or_default()
        } else {
            String::new()
        }
    };
    if mode == "hooked" {
        let mut e = entity.borrow_mut();
        e.y -= 2.0;
        if e.y < -10.0 { e.y = -10.0; }
        return;
    }
    let max_depth = (anim.height() as f64 * 0.75) as i32;
    let mut e = entity.borrow_mut();
    if (e.y as i32) < max_depth {
        e.y += 2.0;
    } else {
        e.y = max_depth as f64;
    }
}

/// Switches an entity into "hooked" upward movement.
/// Used for fish, line, and hook after a catch event.
pub fn retract(entity: EntityRef, _anim: &mut Animation) {
    let mut e = entity.borrow_mut();
    e.physical = false;
    if e.entity_type == "fish" {
        e.z = DEPTH_WATER_GAP2 as f64;
        e.callback = Some(fishhook_callback);
        e.callback_args = CallbackArgs::State({
            let mut m = std::collections::HashMap::new();
            m.insert("mode".into(), "hooked".into());
            m
        });
    } else {
        if e.entity_type == "fishhook" {
            // Only the visible hook should die when it retracts offscreen.
            e.die_offscreen = true;
        }
        e.callback_args = CallbackArgs::State({
            let mut m = std::collections::HashMap::new();
            m.insert("mode".into(), "hooked".into());
            m
        });
    }
}

/// Removes all entities of the listed types from the scene.
/// Used for grouped cleanup (for example hook + line + point).
/// After cleanup, it chains into the next random event.
fn group_death(anim: &mut Animation, bound_types: &[&str]) {
    for tp in bound_types {
        for obj in anim.get_entities_by_type(tp) {
            anim.del_entity(&obj);
        }
    }
    random_object(None, anim);
}

// ── Ducks ─────────────────────────────────────────────────────────────────────

/// Spawns animated ducks on the water surface.
/// They cycle wing/pose frames while moving sideways.
pub fn add_ducks(_dead: Option<EntityRef>, anim: &mut Animation) {
    let mut rng = rand::thread_rng();
    let dir = rng.gen_range(0..2);
    let shapes: [&[&str]; 2] = [
        &[
            "      _          _          _\n,____(')=  ,____(')=  ,____(')<\n \\~~= ')    \\~~= ')    \\~~= ')",
            "      _          _          _\n,____(')=  ,____(')<  ,____(')=\n \\~~= ')    \\~~= ')    \\~~= ')",
            "      _          _          _\n,____(')<  ,____(')=  ,____(')=\n \\~~= ')    \\~~= ')    \\~~= ')",
        ],
        &[
            "  _          _          _\n>(')____,  =(')____,  =(')____,\n (` =~~/    (` =~~/    (` =~~/",
            "  _          _          _\n=(')____,  >(')____,  =(')____,\n (` =~~/    (` =~~/    (` =~~/",
            "  _          _          _\n=(')____,  =(')____,  >(')____,\n (` =~~/    (` =~~/    (` =~~/",
        ],
    ];
    let colors = [
        "      g          g          g\nwwwwwgcgy  wwwwwgcgy  wwwwwgcgy\n wwww Ww    wwww Ww    wwww Ww",
        "  g          g          g\nygcgwwwww  ygcgwwwww  ygcgwwwww\n wW wwww    wW wwww    wW wwww",
    ];
    let speed;
    let x;
    if dir == 0 { speed = 1.0f64;  x = -30i32; }
    else        { speed = -1.0f64; x = anim.width() as i32 - 2; }
    anim.new_entity(EntityOptions {
        shape:          shapes[dir].iter().map(|s| s.to_string()).collect(),
        color:          vec![colors[dir].into()],
        auto_trans:     true,
        position:       [x, 5, DEPTH_WATER_GAP3],
        callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0, 0.25])),
        death_callback: Some(Box::new(|_, a| random_object(None, a))),
        die_offscreen:  true,
        default_color:  "WHITE".into(),
        ..Default::default()
    });
}

// ── Dolphins ──────────────────────────────────────────────────────────────────

/// Spawns three dolphins with fixed spacing.
/// Only the lead dolphin has a death callback to avoid triple respawns.
/// Followers are visual companions in the same formation.
pub fn add_dolphins(_dead: Option<EntityRef>, anim: &mut Animation) {
    let mut rng = rand::thread_rng();
    let dir = rng.gen_range(0..2);
    let shapes: [[&str; 2]; 2] = [
        [
            "        ,\n      __)\\\n(\\_.-'    a`-.\n(/~~````(/~^^`",
            "        ,\n(\\__  __)\\\n(/~.''    a`-.\n    ````\\)~^^`",
        ],
        [
            "     ,\n   _/(__\n.-'a    `-._/)\n'^^~\\)''''~~\\)",
            "     ,\n   _/(__  __/)\n.-'a    ``.~\\)\n'^^~(/''''",
        ],
    ];
    let colors = ["\n\n\n          W", "\n\n\n   W"];
    let speed;
    let x;
    let distance = 15i32;
    if dir == 0 { speed = 2.0f64;  x = -13i32; }
    else        { speed = -2.0f64; x = anim.width() as i32 - 2; }
    let dist_sign = if dir == 0 { distance } else { -distance };

    for i in 0..3usize {
        let default_color = if i < 2 { "BLUE" } else { "CYAN" };
        let death_cb: Option<crate::entity::DeathCallback> = if i == 0 {
            Some(Box::new(|_, a| random_object(None, a)))
        } else {
            None
        };
        anim.new_entity(EntityOptions {
            shape:          shapes[dir].iter().map(|s| s.to_string()).collect(),
            color:          vec![colors[dir].into()],
            auto_trans:     true,
            position:       [x - dist_sign * (2 - i as i32), 5, DEPTH_WATER_GAP3],
            callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0, 0.5])),
            death_callback: death_cb,
            die_offscreen:  true,
            default_color:  default_color.into(),
            ..Default::default()
        });
    }
}

// ── Swan ──────────────────────────────────────────────────────────────────────

/// Spawns a swan gliding near the top water line.
/// Direction and sprite are picked randomly.
pub fn add_swan(_dead: Option<EntityRef>, anim: &mut Animation) {
    let shapes = [
        "       ___\n,_    / _,\\\n| \\   \\( \\|\n|  \\_  \\\\\n(_   \\_) \\\n(\\_   `   \\\n \\   -=~  /",
        " ___\n/,_ \\    _,\n|/ )/   / |\n  //  _/  |\n / ( /   _)\n/   `   _/)\n\\  ~=-   /",
    ];
    let colors = [
        "\n\n         g\n         yy\n\n\n\n",
        "\n\n g\nyy\n\n\n\n",
    ];
    let mut rng = rand::thread_rng();
    let dir = rng.gen_range(0..2);
    let speed;
    let x;
    if dir == 0 { speed = 1.0f64;  x = -10i32; }
    else        { speed = -1.0f64; x = anim.width() as i32 - 2; }
    anim.new_entity(EntityOptions {
        shape:          vec![shapes[dir].into()],
        color:          vec![colors[dir].into()],
        auto_trans:     true,
        position:       [x, 1, DEPTH_WATER_GAP3],
        callback_args:  Some(CallbackArgs::Move(vec![speed, 0.0, 0.0, 0.25])),
        death_callback: Some(Box::new(|_, a| random_object(None, a))),
        die_offscreen:  true,
        default_color:  "WHITE".into(),
        ..Default::default()
    });
}

// ── Router ────────────────────────────────────────────────────────────────────

/// The special-event router for this game.
/// Many death callbacks call this, so events form a continuous chain.
/// Random choice keeps the aquarium from repeating one pattern.
pub fn random_object(dead: Option<EntityRef>, anim: &mut Animation) {
    let choices: &[fn(Option<EntityRef>, &mut Animation)] = &[
        add_ship, add_whale, add_monster, add_big_fish,
        add_shark, add_fishhook, add_swan, add_ducks, add_dolphins,
    ];
    let i = rand::thread_rng().gen_range(0..choices.len());
    choices[i](dead, anim);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::CallbackArgs;
    use std::rc::Rc;
    use std::collections::HashSet;

    fn make_anim() -> Animation {
        let mut a = Animation::new();
        a.width  = 160;
        a.height = 50;
        a
    }

    fn pointers_before(a: &Animation) -> HashSet<*const std::cell::RefCell<crate::entity::Entity>> {
        a.entities.iter().map(|e| Rc::as_ptr(e)).collect()
    }

    fn added_since<'a>(
        a: &'a Animation,
        before: &HashSet<*const std::cell::RefCell<crate::entity::Entity>>,
    ) -> Vec<EntityRef> {
        a.entities.iter()
            .filter(|e| !before.contains(&Rc::as_ptr(e)))
            .cloned()
            .collect()
    }

    // Mirrors Go's TestSeaMonsterSpritesParity.
    #[test]
    fn test_sea_monster_sprites_parity() {
        for _ in 0..40 {
            let mut a = make_anim();
            let before = pointers_before(&a);
            add_monster(None, &mut a);
            let added = added_since(&a, &before);
            assert_eq!(added.len(), 1, "add_monster should add exactly one entity");

            let e = added[0].borrow();
            assert!(e.shapes.len() >= 2,
                "monster must have ≥2 animation frames, got {}", e.shapes.len());
            for frame in &e.shapes {
                assert!(frame.contains('\n'),  "monster frame must be multiline");
                assert!(!frame.contains("\\n"), "monster frame must not contain literal \\n");
            }
            assert!(!e.colors.is_empty() && !e.colors[0].is_empty(),
                "monster must have a color mask");
            assert_eq!(e.colors.len(), e.shapes.len(),
                "color mask count must match shape count");

            if let CallbackArgs::Move(ref v) = e.callback_args {
                assert_eq!(v.len(), 4, "monster callback args should have 4 elements");
                assert_eq!(v[0].abs(), 2.0, "monster speed should be ±2.0");
                assert_eq!(v[1], 0.0);
                assert_eq!(v[2], 0.0);
                assert_eq!(v[3], 0.25, "monster frame step should be 0.25");
            } else {
                panic!("expected Move callback args for monster");
            }
        }
    }

    // Mirrors Go's TestBigFishVisualParity.
    #[test]
    fn test_big_fish_visual_parity() {
        for _ in 0..30 {
            let mut a = Animation::new();
            a.width  = 200;
            a.height = 60;
            let before = pointers_before(&a);
            add_big_fish(None, &mut a);
            let added = added_since(&a, &before);
            assert_eq!(added.len(), 1, "add_big_fish should add exactly one entity");

            let e = added[0].borrow();
            let shape = e.current_shape();
            assert!(shape.contains('\n'),   "big fish shape must be multiline");
            assert!(!shape.contains("\\n"), "big fish must not contain literal \\n");
            assert!(shape.split('\n').count() >= 10,
                "big fish must be ≥10 lines tall");
            assert!(!e.current_color().is_empty(),
                "big fish must have a color mask");

            if let CallbackArgs::Move(ref v) = e.callback_args {
                assert!(!v.is_empty());
                let speed = v[0].abs();
                assert!(speed == 2.5 || speed == 3.0,
                    "big fish speed must be 2.5 or 3.0, got {}", speed);
            } else {
                panic!("expected Move callback args for big fish");
            }
        }
    }

    // Mirrors Go's TestSharkAndTeethParity.
    #[test]
    fn test_shark_and_teeth_parity() {
        let mut a = Animation::new();
        a.width  = 200;
        a.height = 60;
        add_shark(None, &mut a);

        let sharks = a.get_entities_by_type("shark");
        let teeth  = a.get_entities_by_type("teeth");
        assert_eq!(sharks.len(), 1, "expected exactly one shark entity");
        assert_eq!(teeth.len(),  1, "expected exactly one teeth entity");

        let s     = sharks[0].borrow();
        let shape = s.current_shape();
        assert!(shape.contains('\n'),   "shark shape must be multiline");
        assert!(!shape.contains("\\n"), "shark shape must not contain literal \\n");
        assert!(shape.split('\n').count() >= 8,
            "shark sprite must have ≥8 lines");
        assert!(!s.current_color().is_empty(), "shark must have a color mask");

        let (sx, sy, _) = s.position();
        drop(s);
        let (tx, ty, _) = teeth[0].borrow().position();
        assert_eq!(ty, sy + 7, "teeth y must be shark y + 7");
        assert!(tx == -9 || tx == sx + 9,
            "unexpected teeth x: shark_x={} teeth_x={}", sx, tx);
    }

    // Mirrors Go's TestFishhookVisualParity.
    #[test]
    fn test_fishhook_visual_parity() {
        let mut a = Animation::new();
        a.width  = 200;
        a.height = 60;
        add_fishhook(None, &mut a);

        let hooks = a.get_entities_by_type("fishhook");
        let lines = a.get_entities_by_type("fishline");
        assert_eq!(hooks.len(), 1, "expected exactly one fishhook entity");
        assert_eq!(lines.len(), 1, "expected exactly one fishline entity");

        let hook_shape = hooks[0].borrow().current_shape().to_string();
        assert!(hook_shape.split('\n').count() >= 6,
            "fishhook must have ≥6 lines, got {}", hook_shape.split('\n').count());

        let line_shape = lines[0].borrow().current_shape().to_string();
        let pipe_segments = line_shape.match_indices("|\n").count();
        assert!(pipe_segments >= 50,
            "fishline must have ≥50 pipe segments, got {}", pipe_segments);
    }

    // Mirrors Go's TestSurfaceSpritesUseRealMultilineStrings.
    #[test]
    fn test_surface_sprites_use_real_multiline_strings() {
        let mut a = Animation::new();
        a.width  = 120;
        a.height = 40;
        add_ship(None, &mut a);
        add_whale(None, &mut a);
        add_ducks(None, &mut a);
        add_dolphins(None, &mut a);
        add_swan(None, &mut a);

        let surface_depths = [
            crate::depth::DEPTH_WATER_GAP1 as f64,
            crate::depth::DEPTH_WATER_GAP2 as f64,
            crate::depth::DEPTH_WATER_GAP3 as f64,
        ];
        for e in &a.entities {
            let b = e.borrow();
            if !surface_depths.contains(&b.z) { continue; }
            let shape = b.current_shape();
            assert!(shape.contains('\n'),
                "surface sprite {:?} at z={} must be multiline", b.entity_type, b.z);
            assert!(!shape.contains("\\n"),
                "surface sprite {:?} must not contain literal \\n", b.entity_type);
        }
    }

    #[test]
    fn test_dolphins_spawns_three_entities() {
        let mut a = make_anim();
        let before = pointers_before(&a);
        add_dolphins(None, &mut a);
        let added = added_since(&a, &before);
        assert_eq!(added.len(), 3, "add_dolphins should spawn exactly 3 entities");
    }

    #[test]
    fn test_fishhook_lowers_in_lowering_mode() {
        let mut a = Animation::new();
        a.width  = 120;
        a.height = 40;
        add_fishhook(None, &mut a);

        let hook  = a.get_entities_by_type("fishhook")[0].clone();
        let y_before = hook.borrow().y;

        // Call the callback the same way the animate loop does.
        let cb = hook.borrow().callback;
        if let Some(f) = cb { f(hook.clone(), &mut a); }

        let y_after = hook.borrow().y;
        assert!(y_after > y_before,
            "fishhook should descend in lowering mode (y_before={} y_after={})", y_before, y_after);
    }

    #[test]
    fn test_retract_switches_fishhook_to_hooked_mode() {
        let mut a = Animation::new();
        a.width  = 120;
        a.height = 40;
        add_fishhook(None, &mut a);

        let hook = a.get_entities_by_type("fishhook")[0].clone();
        assert!(!hook.borrow().die_offscreen,
            "fishhook should not die offscreen while lowering");
        retract(hook.clone(), &mut a);

        let b = hook.borrow();
        assert!(!b.physical, "retracted entity must not be physical");
        assert!(b.die_offscreen,
            "fishhook should die offscreen after switching to hooked mode");
        if let CallbackArgs::State(ref m) = b.callback_args {
            assert_eq!(m.get("mode").map(String::as_str), Some("hooked"),
                "retract must switch mode to 'hooked'");
        } else {
            panic!("expected State callback args after retract");
        }
    }
}

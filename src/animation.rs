use std::io::{self, BufWriter, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::entity::{Entity, EntityOptions, EntityRef};
use crate::info::info_lines;

/// The main runtime controller for the aquarium.
/// It owns screen state, active entities, and loop flags.
/// Most game flow starts from this structure.
pub struct Animation {
    pub entities: Vec<EntityRef>,
    pub width:    usize,
    pub height:   usize,
    running:      bool,
    stdout:       BufWriter<io::Stdout>,
}

impl Animation {
    /// Creates a new animation manager with defaults.
    /// Call this before spawning anything.
    pub fn new() -> Self {
        Animation {
            entities: Vec::new(),
            width:    0,
            height:   0,
            running:  false,
            stdout:   BufWriter::new(io::stdout()),
        }
    }

    /// Returns current drawable screen width in cells.
    /// Spawners use this value to place entities safely.
    pub fn width(&self)  -> usize { self.width }

    /// Returns current drawable screen height in cells.
    /// This updates after terminal resize events.
    pub fn height(&self) -> usize { self.height }

    /// Builds an entity and adds it to the world.
    /// This helper saves you from calling two functions manually.
    pub fn new_entity(&mut self, opts: EntityOptions) -> EntityRef {
        let e = Entity::new(opts);
        self.add_entity(e.clone());
        e
    }

    /// Adds one entity to the internal list.
    /// It keeps depth order stable for later drawing.
    pub fn add_entity(&mut self, e: EntityRef) {
        self.entities.push(e);
        self.entities.sort_by(|a, b| {
            a.borrow().z.partial_cmp(&b.borrow().z)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Removes one matching entity from the list.
    /// If the entity is missing, this function just returns.
    pub fn del_entity(&mut self, target: &EntityRef) {
        self.entities.retain(|e| !std::rc::Rc::ptr_eq(e, target));
    }

    /// Clears every object from the scene.
    /// This is used by the reset command.
    pub fn remove_all_entities(&mut self) {
        self.entities.clear();
    }

    /// Returns objects with the same type label.
    /// Behavior code uses it to find hooks, teeth, lines, etc.
    pub fn get_entities_by_type(&self, tp: &str) -> Vec<EntityRef> {
        self.entities.iter()
            .filter(|e| e.borrow().entity_type == tp)
            .cloned()
            .collect()
    }

    /// Refreshes width/height after startup or terminal resize.
    /// We require a minimum size so large ASCII art does not break badly.
    /// Height is stored as one row less to avoid bottom-row terminal glitches.
    fn update_size(&mut self, w: u16, h: u16) -> Result<(), String> {
        if h < 15 || w < 40 {
            return Err(format!(
                "terminal too small: need at least 40x15, got {}x{}", w, h
            ));
        }
        self.width  = w as usize;
        self.height = (h - 1) as usize;
        Ok(())
    }

    /// Finds overlaps using simple rectangle checks.
    /// This is an O(n²) pass, but it is easy to understand and fine here.
    /// Results are saved on each entity for later collision handlers.
    fn check_collisions(&mut self) {
        for e in &self.entities {
            e.borrow_mut().collision.clear();
        }
        let n = self.entities.len();
        for i in 0..n {
            let physical = self.entities[i].borrow().physical;
            if !physical { continue; }
            let (ex, ey, ew, eh) = {
                let e = self.entities[i].borrow();
                let (x, y, _) = e.position();
                let (w, h) = e.size();
                (x, y, w as i32, h as i32)
            };
            for j in 0..n {
                if i == j { continue; }
                let (ox, oy, ow, oh) = {
                    let o = self.entities[j].borrow();
                    let (x, y, _) = o.position();
                    let (w, h) = o.size();
                    (x, y, w as i32, h as i32)
                };
                if ex < ox + ow && ex + ew > ox && ey < oy + oh && ey + eh > oy {
                    let other = self.entities[j].clone();
                    self.entities[i].borrow_mut().collision.push(other);
                }
            }
        }
    }

    /// Paints one object frame onto the screen grid.
    /// Shape and color masks are read line-by-line in parallel.
    /// Mask letters/digits pick colors; transparent cells are skipped.
    fn draw_entity(&mut self, eref: &EntityRef) {
        let e = eref.borrow();
        let (ex, ey, _) = e.position();
        let shape       = e.current_shape().to_string();
        let color_mask  = e.current_color().to_string();
        let default_col = color_by_name(&e.default_color);
        let auto_trans  = e.auto_trans;
        let transparent = e.transparent;
        let sw = self.width as i32;
        let sh = self.height as i32;
        drop(e); // release borrow before touching stdout

        let lines:       Vec<&str> = shape.split('\n').collect();
        let color_lines: Vec<&str> = color_mask.split('\n').collect();

        for (li, line) in lines.iter().enumerate() {
            let draw_y = ey + li as i32;
            if draw_y < 0 || draw_y >= sh { continue; }

            let color_chars: Vec<char> = if li < color_lines.len() {
                color_lines[li].chars().collect()
            } else {
                Vec::new()
            };

            let mut last_color: Option<Color> = None;
            for (ci, ch) in line.chars().enumerate() {
                let draw_x = ex + ci as i32;
                if draw_x < 0 || draw_x >= sw { continue; }
                if auto_trans && (ch == ' ' || ch == transparent) { continue; }
                if (ch as u32) < 32 { continue; }

                let col = if ci < color_chars.len() {
                    mask_color(color_chars[ci]).unwrap_or(default_col)
                } else {
                    default_col
                };

                if last_color != Some(col) {
                    let _ = queue!(self.stdout, SetForegroundColor(col));
                    last_color = Some(col);
                }
                let _ = queue!(
                    self.stdout,
                    MoveTo(draw_x as u16, draw_y as u16),
                    Print(ch)
                );
            }
        }
    }

    /// Renders current entities without advancing simulation state.
    /// This is used for immediate redraw requests like terminal resize.
    fn draw_frame(&mut self) {
        let _ = queue!(self.stdout, Clear(ClearType::All));
        let sorted: Vec<EntityRef> = {
            let mut v = self.entities.clone();
            v.sort_by(|a, b| {
                b.borrow().z.partial_cmp(&a.borrow().z)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            v
        };
        for eref in &sorted {
            self.draw_entity(eref);
        }
        let _ = self.stdout.flush();
    }

    /// Rebuilds size-dependent static scenery and keeps dynamic entities.
    /// Also clamps preserved entity Y positions into the drawable vertical range.
    fn reflow_for_resize(&mut self) {
        self.entities.retain(|e| {
            !matches!(e.borrow().entity_type.as_str(), "waterline" | "castle" | "seaweed")
        });
        let h = self.height;
        for eref in &self.entities {
            let mut e = eref.borrow_mut();
            let eh = e.height;
            let max_y = if h > eh { (h - eh) as f64 } else { 0.0 };
            if e.y < 0.0 { e.y = 0.0; }
            if e.y > max_y { e.y = max_y; }
        }
        crate::environment::add_environment(self);
        crate::environment::add_castle(self);
        crate::environment::add_all_seaweed(self);
    }

    /// Shows help text over the aquarium.
    /// It centers lines on screen so controls are easy to read.
    fn draw_info_overlay(&mut self) {
        let _ = queue!(self.stdout, Clear(ClearType::All));
        let lines = info_lines();
        let h = self.height as i32;
        let w = self.width as i32;
        let start_y = ((h - lines.len() as i32) / 2).max(0);
        for (i, ln) in lines.iter().enumerate() {
            let y = start_y + i as i32;
            if y >= h { break; }
            let chars: Vec<char> = ln.chars().collect();
            let x = ((w - chars.len() as i32) / 2).max(0);
            for (ci, &ch) in chars.iter().enumerate() {
                if x + ci as i32 >= w { break; }
                let col = info_color_for(i, ln, ch);
                let _ = queue!(
                    self.stdout,
                    SetForegroundColor(col),
                    MoveTo((x + ci as i32) as u16, y as u16),
                    Print(ch)
                );
            }
        }
        let _ = self.stdout.flush();
    }

    /// Runs one full simulation step and then draws the frame.
    /// We take entity slices before loops so callbacks can add/remove safely.
    /// Pass order is: update → collisions → death cleanup → render.
    fn animate(&mut self) {
        // Movement pass: take entities out so callbacks can call add_entity safely.
        let mut entities = std::mem::take(&mut self.entities);
        for eref in entities.clone() {
            let cb = eref.borrow().callback;
            match cb {
                Some(f) => f(eref.clone(), self),
                None    => Entity::move_entity(eref.clone(), self),
            }
            // Collision handlers use previous frame's data; still valid here.
            let has_coll = {
                let e = eref.borrow();
                e.coll_handler.is_some() && !e.collision.is_empty()
            };
            if has_coll {
                let handler = eref.borrow().coll_handler.unwrap();
                handler(eref.clone(), self);
            }
        }
        // Merge newly spawned entities from callbacks.
        let spawned = std::mem::take(&mut self.entities);
        entities.extend(spawned);
        self.entities = entities;

        self.check_collisions();

        let now = Instant::now();
        let mut i = 0;
        while i < self.entities.len() {
            let dead = {
                let e = self.entities[i].borrow();
                e.should_die(self.width, self.height, now)
            };
            if dead {
                let eref = self.entities.remove(i);
                // death_callback borrows from the entity we just removed; safe.
                let cb = eref.borrow_mut().death_callback.take();
                if let Some(f) = cb {
                    f(eref.clone(), self);
                }
            } else {
                i += 1;
            }
        }

        self.draw_frame();
    }

    /// Initializes terminal raw mode and the alternate screen, then runs the loop.
    /// Always restores the terminal to its original state before returning.
    pub fn run(
        mut self,
        setup: fn(&mut Animation, bool),
        classic: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        terminal::enable_raw_mode()?;
        execute!(self.stdout, EnterAlternateScreen, Hide)?;

        let result = self.run_loop(setup, classic);

        // Always restore terminal.
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, Show);
        let _ = terminal::disable_raw_mode();

        result
    }

    /// The main state machine for the app.
    /// Runs setup, then loops on input and ticks.
    /// Info mode pauses movement so overlay text stays easy to read.
    fn run_loop(
        &mut self,
        setup: fn(&mut Animation, bool),
        classic: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (w, h) = terminal::size()?;
        self.update_size(w, h)?;
        self.running = true;
        setup(self, classic);

        let mut paused      = false;
        let mut showing_info = false;

        while self.running {
            while event::poll(Duration::from_secs(0))? {
                match event::read()? {
                    Event::Resize(w, h) => {
                        self.update_size(w, h)?;
                        self.reflow_for_resize();
                        if showing_info { self.draw_info_overlay(); } else { self.draw_frame(); }
                    }
                    Event::Key(key) => match key.code {
                        KeyCode::Esc if showing_info => {
                            showing_info = false;
                            paused = false;
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            self.running = false;
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            self.remove_all_entities();
                            setup(self, classic);
                        }
                        KeyCode::Char('p') | KeyCode::Char('P') if !showing_info => {
                            paused = !paused;
                        }
                        KeyCode::Char('i') | KeyCode::Char('I') => {
                            showing_info = !showing_info;
                            if showing_info {
                                paused = true;
                                self.draw_info_overlay();
                            } else {
                                paused = false;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            if showing_info {
                self.draw_info_overlay();
            } else if !paused {
                self.animate();
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }
}

/// Converts a color name string to a crossterm Color value.
/// Unknown names are treated as white so drawing still works.
fn color_by_name(name: &str) -> Color {
    match name.to_uppercase().as_str() {
        "BLACK"     => Color::Black,
        "DARK_GREY" => Color::DarkGrey,
        "RED"     => Color::Red,
        "GREEN"   => Color::Green,
        "YELLOW"  => Color::Yellow,
        "BLUE"    => Color::Blue,
        "MAGENTA" => Color::DarkMagenta,
        "CYAN"    => Color::DarkCyan,
        _         => Color::White,
    }
}

/// Maps a single color-mask character to a crossterm Color.
/// Returns None for spaces and non-color characters so the caller falls back to the entity default.
fn mask_color(ch: char) -> Option<Color> {
    match ch {
        'r' | 'R' => Some(Color::Red),
        'g' | 'G' => Some(Color::Green),
        'y' | 'Y' => Some(Color::Yellow),
        'b' | 'B' => Some(Color::Blue),
        'm' | 'M' => Some(Color::DarkMagenta),
        'c' | 'C' => Some(Color::DarkCyan),
        'w' | 'W' => Some(Color::White),
        'k' | 'K' => Some(Color::Black),
        '1'       => Some(Color::DarkCyan),
        '2'       => Some(Color::Yellow),
        '3'       => Some(Color::Green),
        '4'       => Some(Color::White),
        '5'       => Some(Color::Red),
        '6'       => Some(Color::Blue),
        '7'       => Some(Color::DarkMagenta),
        '8'       => Some(Color::Black),
        '9'       => Some(Color::White),
        _         => None,
    }
}

/// Returns a color for one character in the info overlay.
/// It keeps colors consistent across header, controls, and hint lines.
fn info_color_for(line_idx: usize, line: &str, ch: char) -> Color {
    if ch == ' ' { return Color::White; }
    if line_idx <= 4 {
        return if "╔═╗║╚╝".contains(ch) { Color::DarkCyan } else { Color::White };
    }
    if line.contains("Q/q quit")        { return Color::Green; }
    if line.contains("Press I or ESC")  { return Color::DarkMagenta; }
    Color::White
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Entity, EntityOptions};

    fn make_anim() -> Animation {
        let mut a = Animation::new();
        a.width = 120;
        a.height = 40;
        a
    }

    // Populate the scene the same way cli::setup_aquarium does, without calling the
    // private function directly.
    fn setup(a: &mut Animation) {
        crate::environment::add_environment(a);
        crate::environment::add_castle(a);
        crate::environment::add_all_seaweed(a);
        crate::fish::add_all_fish(a, false);
        crate::special::random_object(None, a);
    }

    // Mirrors Go's TestCollisionDetection.
    #[test]
    fn test_collision_detection() {
        let mut a = make_anim();
        let e1 = Entity::new(EntityOptions {
            shape: vec!["xx".into()],
            position: [1, 1, 1],
            physical: true,
            ..Default::default()
        });
        let e2 = Entity::new(EntityOptions {
            shape: vec!["xx".into()],
            position: [2, 1, 1],
            ..Default::default()
        });
        a.entities = vec![e1.clone(), e2.clone()];
        a.check_collisions();
        assert!(!e1.borrow().collision.is_empty(), "expected e1 to detect collision with overlapping e2");
    }

    #[test]
    fn test_no_collision_when_entities_are_apart() {
        let mut a = make_anim();
        let e1 = Entity::new(EntityOptions {
            shape: vec!["xx".into()],
            position: [0, 0, 1],
            physical: true,
            ..Default::default()
        });
        let e2 = Entity::new(EntityOptions {
            shape: vec!["xx".into()],
            position: [10, 0, 1],
            ..Default::default()
        });
        a.entities = vec![e1.clone(), e2.clone()];
        a.check_collisions();
        assert!(e1.borrow().collision.is_empty(), "entities 10 cells apart should not collide");
    }

    #[test]
    fn test_collision_skips_non_physical_entity() {
        let mut a = make_anim();
        let e1 = Entity::new(EntityOptions {
            shape: vec!["xx".into()],
            position: [1, 1, 1],
            physical: false, // non-physical: collision check skipped
            ..Default::default()
        });
        let e2 = Entity::new(EntityOptions {
            shape: vec!["xx".into()],
            position: [1, 1, 1],
            ..Default::default()
        });
        a.entities = vec![e1.clone(), e2.clone()];
        a.check_collisions();
        assert!(e1.borrow().collision.is_empty(), "non-physical entity must not record collisions");
    }

    #[test]
    fn test_get_entities_by_type() {
        let mut a = make_anim();
        a.entities = vec![
            Entity::new(EntityOptions { entity_type: "fish".into(),  shape: vec!["x".into()], ..Default::default() }),
            Entity::new(EntityOptions { entity_type: "shark".into(), shape: vec!["x".into()], ..Default::default() }),
            Entity::new(EntityOptions { entity_type: "fish".into(),  shape: vec!["x".into()], ..Default::default() }),
        ];
        assert_eq!(a.get_entities_by_type("fish").len(), 2);
        assert_eq!(a.get_entities_by_type("shark").len(), 1);
        assert_eq!(a.get_entities_by_type("whale").len(), 0);
    }

    #[test]
    fn test_del_entity_removes_by_pointer() {
        let mut a = make_anim();
        let e1 = Entity::new(EntityOptions { shape: vec!["x".into()], ..Default::default() });
        let e2 = Entity::new(EntityOptions { shape: vec!["x".into()], ..Default::default() });
        a.entities = vec![e1.clone(), e2.clone()];
        a.del_entity(&e1);
        assert_eq!(a.entities.len(), 1);
        assert!(std::rc::Rc::ptr_eq(&a.entities[0], &e2), "e2 should remain after removing e1");
    }

    #[test]
    fn test_add_entity_maintains_ascending_z_order() {
        let mut a = make_anim();
        a.add_entity(Entity::new(EntityOptions { shape: vec!["x".into()], position: [0,0,5], ..Default::default() }));
        a.add_entity(Entity::new(EntityOptions { shape: vec!["x".into()], position: [0,0,1], ..Default::default() }));
        a.add_entity(Entity::new(EntityOptions { shape: vec!["x".into()], position: [0,0,3], ..Default::default() }));
        let zs: Vec<i32> = a.entities.iter().map(|e| e.borrow().z as i32).collect();
        assert_eq!(zs, vec![1, 3, 5], "add_entity should keep entities sorted by ascending z");
    }

    #[test]
    fn test_remove_all_entities_clears_list() {
        let mut a = make_anim();
        a.entities.push(Entity::new(EntityOptions { shape: vec!["x".into()], ..Default::default() }));
        a.entities.push(Entity::new(EntityOptions { shape: vec!["x".into()], ..Default::default() }));
        a.remove_all_entities();
        assert!(a.entities.is_empty(), "remove_all_entities should clear all entities");
    }

    // Mirrors Go's TestReflowForResizePreservesDynamicAndRebuildsStatic.
    #[test]
    fn test_reflow_preserves_dynamic_entities_and_rebuilds_static_scenery() {
        let mut a = make_anim();
        setup(&mut a);

        let fish = Entity::new(EntityOptions {
            entity_type: "fish".into(),
            shape:        vec!["><>".into()],
            position:     [8, 35, crate::depth::DEPTH_FISH_START],
            ..Default::default()
        });
        a.add_entity(fish.clone());

        a.width  = 80;
        a.height = 25;
        a.reflow_for_resize();

        assert!(
            a.entities.iter().any(|e| std::rc::Rc::ptr_eq(e, &fish)),
            "dynamic fish must survive reflow"
        );

        {
            let fb  = fish.borrow();
            let max = (a.height.saturating_sub(fb.height)) as f64;
            assert!(fb.y >= 0.0 && fb.y <= max,
                "fish y={} should be clamped to [0, {}] after reflow", fb.y, max);
        }

        let castles = a.get_entities_by_type("castle");
        assert_eq!(castles.len(), 1, "expected exactly one castle after reflow");
        let (cx, cy, _) = castles[0].borrow().position();
        assert_eq!(cx, a.width as i32 - 32, "castle x wrong after reflow");
        assert_eq!(cy, a.height as i32 - 13, "castle y wrong after reflow");

        assert_eq!(a.get_entities_by_type("waterline").len(), 4,
            "expected 4 waterline entities after reflow");

        let want_seaweed = a.width / 15;
        assert_eq!(a.get_entities_by_type("seaweed").len(), want_seaweed,
            "expected {} seaweed entities after reflow", want_seaweed);
    }
}

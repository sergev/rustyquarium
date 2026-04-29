use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

/// A shared, mutable handle to one entity.
/// `Rc` means multiple parts of the code can hold a reference; `RefCell` lets them mutate it at runtime.
pub type EntityRef = Rc<RefCell<Entity>>;

/// A behavior function for one entity that runs every frame.
/// Plain fn pointer (no captures) used for movement and update logic.
pub type EntityCallback       = fn(EntityRef, &mut crate::animation::Animation);

/// Runs when an entity touches others.
/// Plain fn pointer (no captures) used for hit reactions.
pub type CollisionHandler     = fn(EntityRef, &mut crate::animation::Animation);

/// Runs right before an entity is removed.
/// Stored as a boxed closure so it can capture values like the `classic` flag in fish respawn.
pub type DeathCallback        = Box<dyn Fn(EntityRef, &mut crate::animation::Animation)>;

/// Polymorphic argument bag passed to callbacks.
/// `Move` holds standard movement deltas `[dx, dy, dz, frameStep]`.
/// `State` holds a string key/value map for multi-mode behaviors like the fishhook.
#[derive(Clone)]
pub enum CallbackArgs {
    Move(Vec<f64>),
    State(HashMap<String, String>),
}

/// Stores everything needed for one on-screen object.
/// Includes sprite frames, movement data, collision data, and life rules.
/// Fish, bubbles, hooks, and decorations all use this same base type.
pub struct Entity {
    pub entity_type:   String,
    pub shapes:        Vec<String>,
    pub colors:        Vec<String>,
    pub x:             f64,
    pub y:             f64,
    pub z:             f64,
    pub callback:      Option<EntityCallback>,
    pub callback_args: CallbackArgs,
    pub die_time:      Option<Instant>,
    pub die_offscreen: bool,
    pub die_frame:     i32,
    pub death_callback: Option<DeathCallback>,
    pub default_color: String,
    pub physical:      bool,
    pub coll_handler:  Option<CollisionHandler>,
    pub auto_trans:    bool,
    pub transparent:   char,
    pub current_frame: usize,
    pub frame_time:    f64,
    pub frame_count:   i32,
    pub collision:     Vec<EntityRef>,
    pub alive:         bool,
    pub width:         usize,
    pub height:        usize,
}

/// Input bundle used by `Entity::new`.
/// `shape` holds one or more animation frames; `color` holds matching color masks.
/// Single-frame entities pass a one-element Vec; animated entities pass more.
pub struct EntityOptions {
    pub entity_type:    String,
    pub shape:          Vec<String>,
    pub color:          Vec<String>,
    pub position:       [i32; 3],
    pub callback:       Option<EntityCallback>,
    pub callback_args:  Option<CallbackArgs>,
    pub die_time:       Option<Instant>,
    pub die_offscreen:  bool,
    pub die_frame:      i32,
    pub death_callback: Option<DeathCallback>,
    pub default_color:  String,
    pub physical:       bool,
    pub coll_handler:   Option<CollisionHandler>,
    pub auto_trans:     bool,
}

impl Default for EntityOptions {
    fn default() -> Self {
        Self {
            entity_type:    String::new(),
            shape:          Vec::new(),
            color:          Vec::new(),
            position:       [0, 0, 0],
            callback:       None,
            callback_args:  None,
            die_time:       None,
            die_offscreen:  false,
            die_frame:      0,
            death_callback: None,
            default_color:  String::new(),
            physical:       false,
            coll_handler:   None,
            auto_trans:     false,
        }
    }
}

impl Entity {
    /// Builds an `Entity` from options and fills safe defaults.
    /// Normalizes colors, shape slices, and initial callback arguments.
    /// This gives all entities a consistent starting state.
    pub fn new(opts: EntityOptions) -> EntityRef {
        let mut shapes = opts.shape;
        let mut colors = opts.color;
        if shapes.is_empty() { shapes.push(String::new()); }
        if colors.is_empty() { colors.push(String::new()); }

        let default_color = if opts.default_color.is_empty() {
            "WHITE".to_string()
        } else {
            opts.default_color.to_uppercase()
        };

        let callback_args = opts.callback_args.unwrap_or(CallbackArgs::Move(vec![0.0, 0.0, 0.0, 0.5]));

        let mut e = Entity {
            entity_type:    opts.entity_type,
            shapes,
            colors,
            x:              opts.position[0] as f64,
            y:              opts.position[1] as f64,
            z:              opts.position[2] as f64,
            callback:       opts.callback,
            callback_args,
            die_time:       opts.die_time,
            die_offscreen:  opts.die_offscreen,
            die_frame:      opts.die_frame,
            death_callback: opts.death_callback,
            default_color,
            physical:       opts.physical,
            coll_handler:   opts.coll_handler,
            auto_trans:     opts.auto_trans,
            transparent:    ' ',
            current_frame:  0,
            frame_time:     0.0,
            frame_count:    0,
            collision:      Vec::new(),
            alive:          true,
            width:          0,
            height:         0,
        };
        e.update_dimensions();
        Rc::new(RefCell::new(e))
    }

    /// Recalculates current frame width and height from the first shape.
    /// The renderer and collision checks use these values every frame.
    /// Call it whenever shape data may change.
    pub fn update_dimensions(&mut self) {
        let lines: Vec<&str> = self.shapes[0].split('\n').collect();
        self.height = lines.len();
        self.width  = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    }

    /// Returns integer screen coordinates for drawing and collisions.
    /// The entity stores position as floats internally; this rounds by casting.
    pub fn position(&self) -> (i32, i32, i32) {
        (self.x as i32, self.y as i32, self.z as i32)
    }

    /// Returns current sprite width and height in cells.
    /// This helps clipping and collision math stay simple.
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Picks the frame to draw right now.
    /// It loops automatically when frame index passes frame count.
    pub fn current_shape(&self) -> &str {
        if self.shapes.is_empty() { return ""; }
        &self.shapes[self.current_frame % self.shapes.len()]
    }

    /// Picks the active color-mask frame.
    /// Like shapes, this cycles through available mask frames.
    pub fn current_color(&self) -> &str {
        if self.colors.is_empty() { return ""; }
        &self.colors[self.current_frame % self.colors.len()]
    }

    /// Marks the entity as dead for cleanup.
    /// The animation loop removes dead entities on the next update.
    pub fn kill(&mut self) {
        self.alive = false;
    }

    /// Checks all removal rules for this entity.
    /// It handles manual kill, time/frame limits, and offscreen cleanup.
    /// Returning true means the entity should be deleted now.
    pub fn should_die(&self, screen_w: usize, screen_h: usize, now: Instant) -> bool {
        if !self.alive { return true; }
        if let Some(t) = self.die_time {
            if now >= t { return true; }
        }
        if self.die_frame > 0 && self.frame_count >= self.die_frame {
            return true;
        }
        if self.die_offscreen {
            let x = self.x as i32;
            let y = self.y as i32;
            let w = self.width as i32;
            let h = self.height as i32;
            let sw = screen_w as i32;
            let sh = screen_h as i32;
            if x + w < 0 || x >= sw || y + h < 0 || y >= sh {
                return true;
            }
        }
        false
    }

    /// Default movement logic when no custom callback exists.
    /// For `Move` args, order is `[dx, dy, dz, frameStep]`.
    /// `frameStep` accumulates until >= 1, then the frame index is advanced.
    pub fn move_entity(eref: EntityRef, _anim: &mut crate::animation::Animation) {
        let mut e = eref.borrow_mut();
        let args = e.callback_args.clone();
        match args {
            CallbackArgs::Move(ref v) => {
                if v.len() >= 3 {
                    e.x += v[0];
                    e.y += v[1];
                    e.z += v[2];
                }
                if v.len() >= 4 && v[3] > 0.0 {
                    e.frame_time += v[3];
                    if e.frame_time >= 1.0 {
                        e.current_frame += 1;
                        e.frame_time = 0.0;
                    }
                    e.frame_count += 1;
                }
            }
            _ => {
                if e.shapes.len() > 1 {
                    e.frame_time += 0.1;
                    if e.frame_time >= 1.0 {
                        e.current_frame += 1;
                        e.frame_time = 0.0;
                        e.frame_count += 1;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn make_anim() -> crate::animation::Animation {
        let mut a = crate::animation::Animation::new();
        a.width = 120;
        a.height = 40;
        a
    }

    // Mirrors Go's TestEntityMoveFrame.
    // frameStep 0.6 accumulates: first call gives 0.6 (no advance), second gives 1.2 (advance).
    #[test]
    fn test_entity_move_frame() {
        let mut a = make_anim();
        let e = Entity::new(EntityOptions {
            shape: vec!["a".into(), "b".into()],
            callback_args: Some(CallbackArgs::Move(vec![1.0, 2.0, 0.0, 0.6])),
            ..Default::default()
        });
        Entity::move_entity(e.clone(), &mut a);
        {
            let b = e.borrow();
            assert_eq!(b.x as i32, 1, "x should be 1 after first move");
            assert_eq!(b.y as i32, 2, "y should be 2 after first move");
            assert_eq!(b.current_frame, 0, "frame should not advance on first call");
        }
        Entity::move_entity(e.clone(), &mut a);
        assert_ne!(e.borrow().current_frame, 0, "frame should advance after second call (frame_time crosses 1.0)");
    }

    // Mirrors Go's TestEntityShouldDie.
    #[test]
    fn test_entity_should_die_by_time() {
        let past = Instant::now() - Duration::from_secs(1);
        let e = Entity::new(EntityOptions {
            shape: vec!["abc".into()],
            die_time: Some(past),
            ..Default::default()
        });
        assert!(e.borrow().should_die(100, 100, Instant::now()));
    }

    #[test]
    fn test_entity_should_not_die_with_future_time() {
        let future = Instant::now() + Duration::from_secs(60);
        let e = Entity::new(EntityOptions {
            shape: vec!["abc".into()],
            die_time: Some(future),
            ..Default::default()
        });
        assert!(!e.borrow().should_die(100, 100, Instant::now()));
    }

    #[test]
    fn test_entity_should_not_die_when_alive() {
        let e = Entity::new(EntityOptions {
            shape: vec!["abc".into()],
            ..Default::default()
        });
        assert!(!e.borrow().should_die(100, 100, Instant::now()));
    }

    #[test]
    fn test_entity_should_die_when_killed() {
        let e = Entity::new(EntityOptions {
            shape: vec!["abc".into()],
            ..Default::default()
        });
        e.borrow_mut().kill();
        assert!(e.borrow().should_die(100, 100, Instant::now()));
    }

    #[test]
    fn test_entity_should_die_by_frame_limit() {
        let e = Entity::new(EntityOptions {
            shape: vec!["a".into()],
            die_frame: 3,
            ..Default::default()
        });
        assert!(!e.borrow().should_die(100, 100, Instant::now()), "should not die before frame limit");
        e.borrow_mut().frame_count = 3;
        assert!(e.borrow().should_die(100, 100, Instant::now()), "should die when frame_count reaches die_frame");
    }

    #[test]
    fn test_entity_should_die_offscreen_right() {
        let e = Entity::new(EntityOptions {
            shape: vec!["abc".into()], // width=3
            die_offscreen: true,
            position: [200, 0, 0],
            ..Default::default()
        });
        assert!(e.borrow().should_die(100, 100, Instant::now()), "entity beyond screen right edge should die");
    }

    #[test]
    fn test_entity_should_die_offscreen_left() {
        let e = Entity::new(EntityOptions {
            shape: vec!["abc".into()], // width=3
            die_offscreen: true,
            position: [-10, 0, 0], // x + w = -10 + 3 = -7 < 0
            ..Default::default()
        });
        assert!(e.borrow().should_die(100, 100, Instant::now()), "entity beyond screen left edge should die");
    }

    #[test]
    fn test_entity_position_and_size() {
        let e = Entity::new(EntityOptions {
            shape: vec!["ab\ncd\nef".into()], // 2 wide, 3 tall
            position: [5, 10, 3],
            ..Default::default()
        });
        let b = e.borrow();
        assert_eq!(b.position(), (5, 10, 3));
        assert_eq!(b.size(), (2, 3));
    }

    #[test]
    fn test_entity_frame_cycles_through_shapes() {
        let e = Entity::new(EntityOptions {
            shape: vec!["a".into(), "b".into(), "c".into()],
            ..Default::default()
        });
        assert_eq!(e.borrow().current_shape(), "a");
        e.borrow_mut().current_frame = 2;
        assert_eq!(e.borrow().current_shape(), "c");
        e.borrow_mut().current_frame = 3; // wraps back to index 0
        assert_eq!(e.borrow().current_shape(), "a");
    }

    #[test]
    fn test_entity_default_color_is_uppercased() {
        let e = Entity::new(EntityOptions {
            shape: vec!["x".into()],
            default_color: "cyan".into(),
            ..Default::default()
        });
        assert_eq!(e.borrow().default_color, "CYAN");
    }

    #[test]
    fn test_entity_default_color_falls_back_to_white() {
        let e = Entity::new(EntityOptions {
            shape: vec!["x".into()],
            ..Default::default()
        });
        assert_eq!(e.borrow().default_color, "WHITE");
    }

    #[test]
    fn test_entity_empty_shape_gets_placeholder() {
        let e = Entity::new(EntityOptions { ..Default::default() });
        assert_eq!(e.borrow().shapes.len(), 1, "empty shape list should get one placeholder");
        assert_eq!(e.borrow().current_shape(), "");
    }

    #[test]
    fn test_entity_dimensions_from_multiline_shape() {
        // Verify update_dimensions handles unequal line lengths: width = longest line.
        let e = Entity::new(EntityOptions {
            shape: vec!["hello\nhi".into()], // longest=5, lines=2
            ..Default::default()
        });
        let b = e.borrow();
        assert_eq!(b.width, 5);
        assert_eq!(b.height, 2);
    }
}

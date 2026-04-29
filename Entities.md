# Visual Entity Catalog

## Overview

This document catalogs all visual entities rendered by `rustyquarium`, grouped into classes and described with implementation-level details.

Primary spawn flow:

- Initial scene setup happens in `setup_aquarium` (`src/cli.rs`): environment, castle, seaweed, fish, then one random special object.
- Ongoing special entities are chained through `random_object` (`src/special.rs`) via death callbacks.

An item is treated as a visual entity if it is created through `new_entity`/`add_entity` and rendered by the animation loop (`src/animation.rs`), including composite helper entities such as hit points and linked parts.

---

## Class 1: Environment & Scenery

### `water_seg_0` to `water_seg_3`
- **Class:** Environment & Scenery
- **Defined/Spawned In:** `add_environment` in `src/environment.rs`
- **EntityType / Name:** `entity_type: "waterline"`, `name: "water_seg_<i>"`
- **Visual Form:** Single-frame tiled water bands (`~`/`^` patterns) stretched across screen width.
- **Coloring:** `default_color: "CYAN"`, no explicit color mask.
- **Motion:** Static (no movement callback).
- **Depth Layer:** `DEPTH_WATER_LINE0`..`DEPTH_WATER_LINE3` (from `src/depth.rs`).
- **Lifecycle:** Persistent scene entities.
- **Interactions:** `physical: true`; bubbles collide with `waterline` and pop.

### `castle`
- **Class:** Environment & Scenery
- **Defined/Spawned In:** `add_castle` in `src/environment.rs`
- **EntityType / Name:** `entity_type: "castle"`, `name: "castle"`
- **Visual Form:** Large static multiline ASCII castle.
- **Coloring:** Explicit multiline mask (`R`, `W`, `y`, `w` markers) with `default_color: "BLACK"`.
- **Motion:** Static.
- **Depth Layer:** `DEPTH_CASTLE`.
- **Lifecycle:** Persistent scene entity.
- **Interactions:** Decorative only.

### `seaweed_<rand>`
- **Class:** Environment & Scenery
- **Defined/Spawned In:** `add_seaweed` and `add_all_seaweed` in `src/environment.rs`
- **EntityType / Name:** `entity_type: "seaweed"`, `name: "seaweed_<random>"`
- **Visual Form:** Two-frame alternating plant shape built from `(` and `)` with randomized height (3–6).
- **Coloring:** `default_color: "GREEN"`, no explicit mask.
- **Motion:** Frame animation only using `CallbackArgs::Move([0, 0, 0, frame_step])`.
- **Depth Layer:** `DEPTH_SEAWEED`.
- **Lifecycle:** Timed (`die_time`) and self-regenerating via `death_callback: add_seaweed`.
- **Interactions:** Decorative only.

---

## Class 2: Core Marine Life

### `fish`
- **Class:** Core Marine Life
- **Defined/Spawned In:** `add_fish` and `add_all_fish` in `src/fish.rs`
- **EntityType / Name:** `entity_type: "fish"`
- **Visual Form:** Single-frame per instance; shape selected from directional variants in `OLD_FISH` or `NEW_FISH`.
- **Coloring:** Directional color mask from selected design, randomized by `rand_color`.
- **Motion:** `callback: fish_callback`; primary movement via `CallbackArgs::Move([dx, 0, 0])`.
- **Depth Layer:** Random from `DEPTH_FISH_START` to `DEPTH_FISH_END`.
- **Lifecycle:** `die_offscreen: true`; respawns through a captured-closure death callback to keep population stable.
- **Interactions:** `physical: true`; collides with shark teeth and hook system (`hook_point`).

### Fish Visual Subtypes (`OLD_FISH`, `NEW_FISH`)
- **Class:** Core Marine Life (subtype definitions for `fish`)
- **Defined/Spawned In:** static arrays in `src/fish.rs`, selected in `add_fish`
- **EntityType / Name:** not separate runtime entity types
- **Visual Form:** directional sprite/mask pairs:
  - `OLD_FISH`: 8 variants
  - `NEW_FISH`: 4 variants
- **Coloring:** numeric placeholders (`1..9`) replaced with random color markers (`c/C/r/R/y/Y/b/B/g/G/m/M`).
- **Motion/Lifecycle/Interactions:** inherit `fish` behavior.

### `bubble`
- **Class:** Core Marine Life
- **Defined/Spawned In:** `add_bubble` in `src/fish.rs` (emitted from `fish_callback`)
- **EntityType / Name:** `entity_type: "bubble"`
- **Visual Form:** 5-frame animation: `.`, `o`, `O`, `O`, `O`.
- **Coloring:** `default_color: "CYAN"`.
- **Motion:** `CallbackArgs::Move([0, -1, 0, 0.1])` (rises upward with frame progression).
- **Depth Layer:** Spawned at fish depth minus one (`z - 1`).
- **Lifecycle:** `die_offscreen: true`.
- **Interactions:** `physical: true`; collision handler kills bubble when touching `waterline`.

### `splat` (bite effect)
- **Class:** Core Marine Life (FX)
- **Defined/Spawned In:** `add_splat` in `src/fish.rs` (from `fish_collision`)
- **EntityType / Name:** empty `entity_type`
- **Visual Form:** 4-frame multiline burst effect.
- **Coloring:** `default_color: "RED"`.
- **Motion:** No positional movement; frame stepping only (`CallbackArgs::Move([0,0,0,0.25])`).
- **Depth Layer:** Around bite position, offset to `z - 2`.
- **Lifecycle:** `die_frame: 15` (short-lived).
- **Interactions:** Spawned when small fish are hit by `teeth`.

---

## Class 3: Special Event Creatures

### `ship`
- **Class:** Special Event Creatures
- **Defined/Spawned In:** `add_ship` in `src/special.rs`
- **EntityType / Name:** empty `entity_type`
- **Visual Form:** Single-frame (directional) sailboat sprite.
- **Coloring:** Directional color mask for sails/hull, `default_color: "WHITE"`.
- **Motion:** Horizontal drift via `CallbackArgs::Move([±1, 0, 0, 0])`.
- **Depth Layer:** `DEPTH_WATER_GAP1`.
- **Lifecycle:** `die_offscreen: true`, chains into `random_object`.
- **Interactions:** Decorative moving special.

### `whale`
- **Class:** Special Event Creatures
- **Defined/Spawned In:** `add_whale` in `src/special.rs`
- **EntityType / Name:** empty `entity_type`
- **Visual Form:** 12 frames (5 idle + 7 spout frames), directional.
- **Coloring:** Directional mask reused across frames, `default_color: "WHITE"`.
- **Motion:** `CallbackArgs::Move([±0.5, 0, 0, 1])`.
- **Depth Layer:** `DEPTH_WATER_GAP2`.
- **Lifecycle:** `die_offscreen: true`, chains into `random_object`.
- **Interactions:** Decorative moving special.

### `monster` (new/old variants)
- **Class:** Special Event Creatures
- **Defined/Spawned In:** `add_monster`, `add_new_monster`, `add_old_monster` in `src/special.rs`
- **EntityType / Name:** empty `entity_type`
- **Visual Form:** Two monster families:
  - new monster: 2-frame animation
  - old monster: 4-frame animation
- **Coloring:** Eye highlight masks repeated per frame, `default_color: "GREEN"`.
- **Motion:** `CallbackArgs::Move([±2, 0, 0, 0.25])`.
- **Depth Layer:** `DEPTH_WATER_GAP2`.
- **Lifecycle:** `die_offscreen: true`, chains into `random_object`.
- **Interactions:** Event creature in random-object rotation.

### `big fish` (design1/design2 variants)
- **Class:** Special Event Creatures
- **Defined/Spawned In:** `add_big_fish`, `add_big_fish1`, `add_big_fish2` in `src/special.rs`
- **EntityType / Name:** empty `entity_type`
- **Visual Form:** Large single-frame special fish with directional versions.
- **Coloring:** Large masks passed through `rand_color`, `default_color: "YELLOW"`.
- **Motion:** Horizontal movement:
  - design1: `CallbackArgs::Move([±3.0, 0, 0])`
  - design2: `CallbackArgs::Move([±2.5, 0, 0])`
- **Depth Layer:** `DEPTH_SHARK`.
- **Lifecycle:** `die_offscreen: true`, chains into `random_object`.
- **Interactions:** Event creature in random-object rotation.

### `swan`
- **Class:** Special Event Creatures
- **Defined/Spawned In:** `add_swan` in `src/special.rs`
- **EntityType / Name:** empty `entity_type`
- **Visual Form:** Single-frame directional sprite.
- **Coloring:** Directional accent masks, `default_color: "WHITE"`.
- **Motion:** `CallbackArgs::Move([±1, 0, 0, 0.25])`.
- **Depth Layer:** `DEPTH_WATER_GAP3`.
- **Lifecycle:** `die_offscreen: true`, chains into `random_object`.
- **Interactions:** Decorative moving special.

### `ducks`
- **Class:** Special Event Creatures
- **Defined/Spawned In:** `add_ducks` in `src/special.rs`
- **EntityType / Name:** empty `entity_type`
- **Visual Form:** 3-frame directional animated flock sprite.
- **Coloring:** Directional masks, `default_color: "WHITE"`.
- **Motion:** `CallbackArgs::Move([±1, 0, 0, 0.25])`.
- **Depth Layer:** `DEPTH_WATER_GAP3`.
- **Lifecycle:** `die_offscreen: true`, chains into `random_object`.
- **Interactions:** Decorative moving special.

### `dolphins` (formation)
- **Class:** Special Event Creatures
- **Defined/Spawned In:** `add_dolphins` in `src/special.rs`
- **EntityType / Name:** empty `entity_type`
- **Visual Form:** Three separate entities in fixed spacing, each 2-frame directional animation.
- **Coloring:** Shared directional mask; defaults are `BLUE`, `BLUE`, and `CYAN` by formation index.
- **Motion:** `CallbackArgs::Move([±2, 0, 0, 0.5])`.
- **Depth Layer:** `DEPTH_WATER_GAP3`.
- **Lifecycle:** `die_offscreen: true`; only lead dolphin (index 0) has a death callback to `random_object`.
- **Interactions:** Composite visual formation.

---

## Class 4: Interaction & Composite Systems

### Shark Composite System (`shark` + `teeth`)

#### `shark`
- **Class:** Interaction & Composite Systems
- **Defined/Spawned In:** `add_shark` in `src/special.rs`
- **EntityType / Name:** `entity_type: "shark"`
- **Visual Form:** Large directional single-frame predator sprite.
- **Coloring:** Directional mask with `default_color: "CYAN"`.
- **Motion:** `CallbackArgs::Move([±2, 0, 0])`.
- **Depth Layer:** `DEPTH_SHARK`.
- **Lifecycle:** `die_offscreen: true`; `death_callback: shark_death`.
- **Interactions:** Linked to `teeth` entity for collision-driven predation.

#### `teeth`
- **Class:** Interaction & Composite Systems
- **Defined/Spawned In:** `add_shark` in `src/special.rs`
- **EntityType / Name:** `entity_type: "teeth"`
- **Visual Form:** Single `*` hitbox marker.
- **Coloring:** Default renderer color fallback (no explicit mask or default color override).
- **Motion:** `CallbackArgs::Move([±2, 0, 0])`, matched with shark speed.
- **Depth Layer:** `DEPTH_SHARK + 1`.
- **Lifecycle:** Removed by `shark_death`.
- **Interactions:** `fish` collision handler uses `teeth` hits to trigger fish death and `splat`.

### Fishhook Composite System (`fishline` + `fishhook` + `hook_point`)

#### `fishline`
- **Class:** Interaction & Composite Systems
- **Defined/Spawned In:** `add_fishhook` in `src/special.rs`
- **EntityType / Name:** `entity_type: "fishline"`
- **Visual Form:** Tall multiline line (`|\n` repeated with trailing blank tail).
- **Coloring:** Default renderer color fallback.
- **Motion:** `callback: fishhook_callback` with `CallbackArgs::State({"mode": "lowering"/"hooked"})`.
- **Depth Layer:** `DEPTH_WATER_LINE1`.
- **Lifecycle:** Removed by grouped death cleanup.
- **Interactions:** Retracted when fish is hooked.

#### `fishhook`
- **Class:** Interaction & Composite Systems
- **Defined/Spawned In:** `add_fishhook` in `src/special.rs`
- **EntityType / Name:** `entity_type: "fishhook"`
- **Visual Form:** Multi-line hook sprite.
- **Coloring:** `default_color: "GREEN"`.
- **Motion:** `callback: fishhook_callback` with `CallbackArgs::State` args.
- **Depth Layer:** `DEPTH_WATER_LINE1`.
- **Lifecycle:** `die_offscreen: true`; death callback removes `hook_point` and `fishline` via `group_death`.
- **Interactions:** Part of hook system that can catch fish.

#### `hook_point`
- **Class:** Interaction & Composite Systems
- **Defined/Spawned In:** `add_fishhook` in `src/special.rs`
- **EntityType / Name:** `entity_type: "hook_point"`
- **Visual Form:** Tiny 4-line marker used as collision point.
- **Coloring:** `default_color: "GREEN"`.
- **Motion:** `callback: fishhook_callback` with `CallbackArgs::State` args.
- **Depth Layer:** `DEPTH_SHARK + 1`.
- **Lifecycle:** Removed in grouped hook cleanup.
- **Interactions:** `physical: true`; fish collision with `hook_point` initiates retract sequence.

---

## Appendix A: Depth & Layer Reference

From `src/depth.rs`:
- `DEPTH_GUI_TEXT: 0`, `DEPTH_GUI: 1`
- `DEPTH_SHARK: 2`
- `DEPTH_FISH_START: 3` to `DEPTH_FISH_END: 20`
- `DEPTH_SEAWEED: 21`, `DEPTH_CASTLE: 22`
- Water bands and gaps:
  - `DEPTH_WATER_LINE3: 2`, `DEPTH_WATER_GAP3: 3`
  - `DEPTH_WATER_LINE2: 4`, `DEPTH_WATER_GAP2: 5`
  - `DEPTH_WATER_LINE1: 6`, `DEPTH_WATER_GAP1: 7`
  - `DEPTH_WATER_LINE0: 8`, `DEPTH_WATER_GAP0: 9`

The renderer sorts by depth and draws in descending order (`src/animation.rs`), so higher depth values appear in front.

## Appendix B: Shared Rendering & Animation Rules

- **Frame selection:** `Entity.current_frame` indexes `shapes`/`colors` cyclically (`src/entity.rs`).
- **Movement callback args convention:** `CallbackArgs::Move(vec![dx, dy, dz, frame_step])`.
- **Auto transparency:** with `auto_trans: true`, space characters are skipped in the draw path (`src/animation.rs`).
- **Color masks:** `current_color` lines map per-character mask markers to crossterm `Color` values through `mask_color` (`src/animation.rs`).
- **Entity lifecycle controls:** `die_offscreen`, `die_frame`, `die_time`, explicit `kill()`, and `death_callback`.

## Appendix C: Completeness Cross-Check

Cataloged visual entities instantiated from:

- `src/environment.rs`: `waterline` segments, castle, seaweed
- `src/fish.rs`: fish, bubble, splat
- `src/special.rs`: ship, whale, monster variants, big fish variants, shark system, fishhook system, swan, ducks, dolphins

Supporting behavior/layer references verified against:

- `src/cli.rs`, `src/depth.rs`, `src/entity.rs`, `src/animation.rs`

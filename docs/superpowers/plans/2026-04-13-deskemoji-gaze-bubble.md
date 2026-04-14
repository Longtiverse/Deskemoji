# Deskemoji Gaze Bubble Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add one shippable interaction pass where `thinking` uses cleaner eye-follow frame variants and `hot` can show a short anchored resource bubble in the native Deskemoji window.

**Architecture:** Keep the existing sprite-based renderer and add two narrowly scoped capabilities: state-aware variant selection for `thinking`, and a lightweight bubble overlay renderer for short status text like `CPU 99%`. Avoid any fake eye overlay layers in the runtime path; use asset variants plus existing motion transforms.

**Tech Stack:** Rust, `winit`, `softbuffer`, existing PNG asset pipeline, tests under `tests/`

---

## File Structure

**Create**

- `D:/Project/Deskemoji/tests/gaze_bubble_test.rs`

**Modify**

- `D:/Project/Deskemoji/src/emoji_assets.rs`
- `D:/Project/Deskemoji/src/main.rs`
- `D:/Project/Deskemoji/src/renderer.rs`
- `D:/Project/Deskemoji/src/lib.rs`
- `D:/Project/Deskemoji/tests/render_math_test.rs`

### Task 1: Add Test Coverage For Gaze Variant Selection

**Files:**
- Create: `D:/Project/Deskemoji/tests/gaze_bubble_test.rs`
- Modify: `D:/Project/Deskemoji/src/main.rs`

- [ ] **Step 1: Write the failing test**

```rust
use deskemoji::renderer::GazeDirection;

#[test]
fn choose_gaze_direction_prefers_cardinal_and_diagonal_variants() {
    assert_eq!(GazeDirection::from_pointer_delta(-80.0, -80.0), GazeDirection::UpLeft);
    assert_eq!(GazeDirection::from_pointer_delta(0.0, -90.0), GazeDirection::Up);
    assert_eq!(GazeDirection::from_pointer_delta(90.0, 0.0), GazeDirection::Right);
    assert_eq!(GazeDirection::from_pointer_delta(0.0, 0.0), GazeDirection::Center);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test choose_gaze_direction_prefers_cardinal_and_diagonal_variants -- --nocapture`

Expected: FAIL because `GazeDirection` and mapping logic do not exist yet.

- [ ] **Step 3: Implement the minimal gaze direction type and mapping**

Add:

- `renderer::GazeDirection`
- `GazeDirection::from_pointer_delta(dx, dy)`

Keep thresholds conservative so gaze only changes when pointer movement is meaningfully directional.

- [ ] **Step 4: Re-run the test and verify it passes**

Run: `cargo test choose_gaze_direction_prefers_cardinal_and_diagonal_variants -- --nocapture`

Expected: PASS

### Task 2: Add Test Coverage For Bubble Layout

**Files:**
- Modify: `D:/Project/Deskemoji/tests/render_math_test.rs`
- Modify: `D:/Project/Deskemoji/src/renderer.rs`

- [ ] **Step 1: Write the failing test**

```rust
use deskemoji::renderer::{compute_bubble_rect, BubbleRect};

#[test]
fn compute_bubble_rect_places_bubble_up_left_of_sprite_anchor() {
    let bubble = compute_bubble_rect(120, 120, 96, 84, 72, 34);

    assert!(bubble.left < 96);
    assert!(bubble.top < 84);
    assert!(bubble.tail_tip_x > bubble.left);
    assert!(bubble.tail_tip_y > bubble.top);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test compute_bubble_rect_places_bubble_up_left_of_sprite_anchor -- --nocapture`

Expected: FAIL because bubble geometry does not exist yet.

- [ ] **Step 3: Implement minimal bubble layout helpers**

Add:

- `renderer::BubbleRect`
- `renderer::compute_bubble_rect(...)`

The first version only needs:

- rounded body rect
- tail tip anchor toward sprite upper-right area
- short text fitting for one compact label

- [ ] **Step 4: Re-run the test and verify it passes**

Run: `cargo test compute_bubble_rect_places_bubble_up_left_of_sprite_anchor -- --nocapture`

Expected: PASS

### Task 3: Wire Thinking Variants Into Emoji Assets

**Files:**
- Modify: `D:/Project/Deskemoji/src/emoji_assets.rs`
- Create: `D:/Project/Deskemoji/tests/gaze_bubble_test.rs`

- [ ] **Step 1: Write the failing test**

```rust
use deskemoji::emoji_assets::{EmojiAssets, EmojiId};
use deskemoji::renderer::GazeDirection;

#[test]
fn thinking_state_exposes_frame_variants() {
    let assets = EmojiAssets::load_from_dir("assets/emoji").unwrap();

    assert!(assets.get_variant(EmojiId::Thinking, GazeDirection::Center).is_some());
    assert!(assets.get_variant(EmojiId::Thinking, GazeDirection::Left).is_some());
    assert!(assets.get_variant(EmojiId::Thinking, GazeDirection::UpRight).is_some());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test thinking_state_exposes_frame_variants -- --nocapture`

Expected: FAIL because per-direction variant loading is not implemented yet.

- [ ] **Step 3: Implement minimal variant lookup**

Add support for:

- `thinking-center.png`
- `thinking-left.png`
- `thinking-right.png`
- `thinking-up.png`
- `thinking-down.png`
- `thinking-ul.png`
- `thinking-ur.png`
- `thinking-dl.png`
- `thinking-dr.png`

Expose:

- `EmojiAssets::get_variant(id, direction)`
- fallback to base image for states without variants

- [ ] **Step 4: Re-run the test and verify it passes**

Run: `cargo test thinking_state_exposes_frame_variants -- --nocapture`

Expected: PASS

### Task 4: Render Short Bubble Overlay

**Files:**
- Modify: `D:/Project/Deskemoji/src/renderer.rs`
- Modify: `D:/Project/Deskemoji/tests/render_math_test.rs`

- [ ] **Step 1: Write the failing test**

```rust
use deskemoji::renderer::bubble_label_for_usage;

#[test]
fn bubble_label_is_compact_ascii() {
    assert_eq!(bubble_label_for_usage("CPU", 99), "CPU 99%");
    assert_eq!(bubble_label_for_usage("MEM", 89), "MEM 89%");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test bubble_label_is_compact_ascii -- --nocapture`

Expected: FAIL because label formatting does not exist yet.

- [ ] **Step 3: Implement minimal bubble rendering**

Add:

- compact label formatter
- lightweight rounded bubble drawing
- minimal bitmap text drawing for uppercase ASCII and digits

Keep the label single-line only.

- [ ] **Step 4: Re-run the test and verify it passes**

Run: `cargo test bubble_label_is_compact_ascii -- --nocapture`

Expected: PASS

### Task 5: Hook The Feature Into Native Runtime

**Files:**
- Modify: `D:/Project/Deskemoji/src/main.rs`
- Modify: `D:/Project/Deskemoji/src/renderer.rs`

- [ ] **Step 1: Write the failing integration test**

```rust
use deskemoji::renderer::{BubbleOverlay, GazeDirection};

#[test]
fn non_hot_states_do_not_emit_resource_bubble() {
    assert!(BubbleOverlay::for_state("happy", 24, 48).is_none());
    assert!(BubbleOverlay::for_state("thinking", 42, 58).is_none());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test non_hot_states_do_not_emit_resource_bubble -- --nocapture`

Expected: FAIL because bubble state logic does not exist yet.

- [ ] **Step 3: Implement the minimal runtime wiring**

In `main.rs`:

- track pointer delta for gaze selection while hovered
- use gaze variants only for `EmojiId::Thinking`
- emit short bubble overlay only for `EmojiId::Hot`

In `renderer.rs`:

- render sprite first
- render bubble second when present

- [ ] **Step 4: Re-run targeted tests**

Run: `cargo test --test gaze_bubble_test -- --nocapture`

Expected: PASS

- [ ] **Step 5: Run the focused binary check**

Run: `cargo check --bin deskemoji`

Expected: PASS

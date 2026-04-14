# Native Emoji Rendering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current hand-drawn emoji renderer with a fixed native-emoji asset pipeline so Deskemoji matches the approved emoji style while keeping only light transform-based animations.

**Architecture:** Introduce an emoji asset catalog that maps the 8 app states to decoded PNG resources, then simplify the runtime animation model so the renderer composes a selected image with float/tap/transition transforms instead of redrawing facial features. Retire eye-tracking/blink-by-redraw behavior and move the renderer toward sprite compositing over a transparent softbuffer surface.

**Tech Stack:** Rust, `winit`, `softbuffer`, `image`, existing asset folder under `assets/emoji`

---

## File Structure

**Create**

- `D:/Project/Deskemoji/src/emoji_assets.rs`
- `D:/Project/Deskemoji/tests/emoji_assets_test.rs`
- `D:/Project/Deskemoji/tests/render_math_test.rs`

**Modify**

- `D:/Project/Deskemoji/src/main.rs`
- `D:/Project/Deskemoji/src/renderer.rs`
- `D:/Project/Deskemoji/Cargo.toml`
- `D:/Project/Deskemoji/assets/emoji/README.md`

**Responsibility split**

- `src/main.rs`
  - owns app state, state-to-emoji selection, and simple animation timing
- `src/emoji_assets.rs`
  - owns emoji IDs, file mapping, PNG decode/loading, and lookup APIs
- `src/renderer.rs`
  - owns surface resize, frame clearing, transform math, and emoji image compositing
- `tests/emoji_assets_test.rs`
  - verifies correct state mapping and asset loading behavior
- `tests/render_math_test.rs`
  - verifies transform math stays stable across animation cases

---

### Task 1: Lock The Approved Emoji Set

**Files:**

- Modify: `D:/Project/Deskemoji/src/main.rs`
- Create: `D:/Project/Deskemoji/tests/emoji_assets_test.rs`

- [ ] **Step 1: Write the failing test for approved emoji mapping**

```rust
use deskemoji::emoji_assets::EmojiId;

#[test]
fn approved_emoji_ids_match_the_user_confirmed_set() {
    let expected = [
        EmojiId::Happy,
        EmojiId::Sad,
        EmojiId::Angry,
        EmojiId::Sleepy,
        EmojiId::Thinking,
        EmojiId::Hot,
        EmojiId::Mindblown,
        EmojiId::Goodnight,
    ];

    assert_eq!(EmojiId::all(), expected);
    assert_eq!(EmojiId::Happy.emoji_char(), '🙂');
    assert_eq!(EmojiId::Sad.emoji_char(), '😢');
    assert_eq!(EmojiId::Angry.emoji_char(), '😠');
    assert_eq!(EmojiId::Sleepy.emoji_char(), '😴');
    assert_eq!(EmojiId::Thinking.emoji_char(), '🤔');
    assert_eq!(EmojiId::Hot.emoji_char(), '🥵');
    assert_eq!(EmojiId::Mindblown.emoji_char(), '🤯');
    assert_eq!(EmojiId::Goodnight.emoji_char(), '😌');
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test approved_emoji_ids_match_the_user_confirmed_set -- --nocapture`

Expected: FAIL because `emoji_assets` and `EmojiId` do not exist yet, or the current mapping still points at the wrong characters.

- [ ] **Step 3: Add the minimal approved emoji enum and state mapping**

Implement in `src/emoji_assets.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmojiId {
    Happy,
    Sad,
    Angry,
    Sleepy,
    Thinking,
    Hot,
    Mindblown,
    Goodnight,
}
```

Add helper methods:

- `EmojiId::all() -> [EmojiId; 8]`
- `EmojiId::emoji_char() -> char`
- `EmojiId::file_stem() -> &'static str`

Update `src/main.rs` to use `EmojiId` rather than raw `EMOJIS` string tuples as the long-term source of truth.

- [ ] **Step 4: Re-run the test and verify it passes**

Run: `cargo test approved_emoji_ids_match_the_user_confirmed_set -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/emoji_assets.rs tests/emoji_assets_test.rs
git commit -m "refactor: define approved native emoji state catalog"
```

---

### Task 2: Wire Real Asset Loading

**Files:**

- Modify: `D:/Project/Deskemoji/Cargo.toml`
- Modify: `D:/Project/Deskemoji/src/emoji_assets.rs`
- Create: `D:/Project/Deskemoji/tests/emoji_assets_test.rs`
- Modify: `D:/Project/Deskemoji/assets/emoji/README.md`

- [ ] **Step 1: Write the failing test for PNG asset loading**

```rust
use deskemoji::emoji_assets::{EmojiAssets, EmojiId};

#[test]
fn loads_png_asset_for_each_approved_state() {
    let assets = EmojiAssets::load_from_dir("assets/emoji").unwrap();

    for id in EmojiId::all() {
        let image = assets.get(id).unwrap();
        assert!(image.width > 0);
        assert!(image.height > 0);
        assert_eq!(image.pixels.len(), (image.width * image.height * 4) as usize);
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test loads_png_asset_for_each_approved_state -- --nocapture`

Expected: FAIL because the runtime asset loader does not exist yet.

- [ ] **Step 3: Implement minimal asset loading with `image`**

Implement in `src/emoji_assets.rs`:

- `struct EmojiImage { width: u32, height: u32, pixels: Vec<u8> }`
- `struct EmojiAssets { ... }`
- `EmojiAssets::load_from_dir(path: impl AsRef<Path>) -> Result<Self, String>`
- `EmojiAssets::get(id: EmojiId) -> Option<&EmojiImage>`

Load these filenames:

- `happy.png`
- `sad.png`
- `angry.png`
- `sleepy.png`
- `thinking.png`
- `hot.png`
- `mindblown.png`
- `goodnight.png`

Document in `assets/emoji/README.md` that these files are now runtime assets, not just placeholder references.

- [ ] **Step 4: Re-run the test and verify it passes**

Run: `cargo test loads_png_asset_for_each_approved_state -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/emoji_assets.rs tests/emoji_assets_test.rs assets/emoji/README.md
git commit -m "feat: load native emoji png assets"
```

---

### Task 3: Replace Facial Drawing With Sprite Composition

**Files:**

- Modify: `D:/Project/Deskemoji/src/renderer.rs`
- Create: `D:/Project/Deskemoji/tests/render_math_test.rs`

- [ ] **Step 1: Write the failing test for destination-rect transform math**

```rust
use deskemoji::renderer::compute_sprite_rect;

#[test]
fn compute_sprite_rect_keeps_emoji_centered_after_scale_and_bounce() {
    let rect = compute_sprite_rect(120, 120, 60.0, 64.0, 1.1, -6.0);

    assert_eq!(rect.center_x, 60);
    assert!(rect.top < 64);
    assert!(rect.width > 0);
    assert!(rect.height > 0);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test compute_sprite_rect_keeps_emoji_centered_after_scale_and_bounce -- --nocapture`

Expected: FAIL because helper math does not exist yet.

- [ ] **Step 3: Implement minimal sprite compositing path**

In `src/renderer.rs`:

- add a small helper for sprite destination math
- remove the giant per-face draw-function switch as the main runtime path
- add a compositing function that copies decoded RGBA asset pixels into the softbuffer frame
- support only:
  - centered placement
  - uniform scale
  - vertical offset
  - optional alpha for transitions

Keep the first implementation simple:

- nearest-neighbor or straightforward integer sampling is acceptable for the first pass
- correctness and visual alignment matter more than fancy filtering

- [ ] **Step 4: Re-run the tests and verify they pass**

Run: `cargo test compute_sprite_rect_keeps_emoji_centered_after_scale_and_bounce -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/renderer.rs tests/render_math_test.rs
git commit -m "refactor: compose emoji sprites instead of hand-drawing faces"
```

---

### Task 4: Simplify Runtime Animation Model Around The Asset

**Files:**

- Modify: `D:/Project/Deskemoji/src/main.rs`
- Modify: `D:/Project/Deskemoji/src/renderer.rs`

- [ ] **Step 1: Write the failing test for animation mode outputs**

If logic is extracted into helpers, add a test like:

```rust
#[test]
fn idle_animation_outputs_small_transform_values() {
    let transform = compute_idle_transform(1.25);
    assert!(transform.offset_y.abs() <= 6.0);
    assert!(transform.scale > 0.95);
    assert!(transform.scale < 1.08);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test idle_animation_outputs_small_transform_values -- --nocapture`

Expected: FAIL because the helper does not exist yet.

- [ ] **Step 3: Replace face-specific animation state with transform animation**

In `src/main.rs`:

- remove or stop using:
  - `eye_x`
  - `eye_y`
  - blink state that redraws eyes
- keep and adapt:
  - bounce/tap timing
  - hover breathing timing
  - animation clock

Add transform-oriented helpers for:

- idle float
- tap squash/rebound
- state transition alpha/scale blend

Pass only transform data plus `EmojiId` into the renderer.

- [ ] **Step 4: Re-run the test and verify it passes**

Run: `cargo test idle_animation_outputs_small_transform_values -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/renderer.rs
git commit -m "refactor: drive emoji presentation with transform animations"
```

---

### Task 5: Integrate Assets Into The App Render Loop

**Files:**

- Modify: `D:/Project/Deskemoji/src/main.rs`
- Modify: `D:/Project/Deskemoji/src/renderer.rs`
- Modify: `D:/Project/Deskemoji/src/emoji_assets.rs`

- [ ] **Step 1: Write the failing smoke test for state lookup**

```rust
#[test]
fn app_state_resolves_to_a_renderable_emoji_asset() {
    let assets = EmojiAssets::load_from_dir("assets/emoji").unwrap();

    for id in EmojiId::all() {
        assert!(assets.get(id).is_some());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails if integration is incomplete**

Run: `cargo test app_state_resolves_to_a_renderable_emoji_asset -- --nocapture`

Expected: FAIL until the app and renderer share the real catalog cleanly.

- [ ] **Step 3: Complete render-loop integration**

Make `App::render()`:

- resolve current state to `EmojiId`
- look up the corresponding loaded asset
- compute presentation transform
- call renderer composition with that asset

Make renderer initialization own or receive the loaded asset catalog in a stable way.

- [ ] **Step 4: Run targeted verification**

Run:

```bash
cargo test emoji_assets_test -- --nocapture
cargo test render_math_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/renderer.rs src/emoji_assets.rs tests/emoji_assets_test.rs tests/render_math_test.rs
git commit -m "feat: render approved emoji assets in the desktop widget"
```

---

### Task 6: Remove Dead Hand-Drawn Paths And Verify Build Health

**Files:**

- Modify: `D:/Project/Deskemoji/src/renderer.rs`
- Modify: `D:/Project/Deskemoji/Cargo.toml`

- [ ] **Step 1: Delete no-longer-used hand-drawn helpers**

Remove dead functions/constants that only existed for procedural face drawing once the sprite path is fully live.

- [ ] **Step 2: Verify the dependency story**

Check:

- `image` remains because asset decoding is now real runtime behavior
- no unused face-drawing constants remain

- [ ] **Step 3: Run final verification**

Run:

```bash
cargo test -- --nocapture
cargo check
```

Expected:

- tests pass
- check succeeds if disk pressure is manageable

If `cargo check` still fails for environment-only disk-space reasons, record that explicitly in the handoff.

- [ ] **Step 4: Commit**

```bash
git add src/renderer.rs Cargo.toml
git commit -m "chore: remove obsolete hand-drawn renderer paths"
```

---

## Notes For Execution

- Keep animation amplitudes small. The user approved the native emoji look, not exaggerated character acting.
- Do not reintroduce custom eye tracking or procedural blinking unless it can be layered externally without altering the approved face.
- If the current assets in `assets/emoji` do not actually match the approved preview style, pause and replace/refresh the asset source before completing runtime integration.
- The current repo has shown disk-space-related verification pain. Prefer targeted tests early, then run the full suite at the end.

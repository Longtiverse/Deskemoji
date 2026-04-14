# Native Emoji Rendering Design

**Date:** 2026-04-11

## Goal

Replace the current hand-drawn emoji face renderer with a stable native-emoji-based presentation model so Deskemoji looks like the real small emoji references the user approved, while still supporting light, smooth desktop animations.

## Decision

Deskemoji will stop using custom-drawn facial features as the primary visual source.

Instead, each app state will map to a fixed emoji resource or native emoji glyph presentation that matches the approved reference style:

- 开心: `🙂`
- 难过: `😢`
- 生气: `😠`
- 困倦: `😴`
- 思考: `🤔`
- 热: `🥵`
- 崩溃: `🤯`
- 晚安: `😌`

The core requirement is visual fidelity to the approved emoji look, not procedural drawing purity.

## Why This Direction

The user explicitly rejected multiple hand-drawn previews, even when they were closer to Fluent styling.

What finally matched expectations was the version that used the real emoji glyph itself. That means the product requirement is:

1. The large Deskemoji face must look like the small approved emoji.
2. Animation must preserve that look instead of redrawing or reinterpreting the face.
3. Rendering consistency is more important than custom illustration flexibility.

## Non-Goals

This design does not try to:

- create a custom house illustration style
- procedurally mimic Apple/Fluent/Noto faces by hand
- support arbitrary emoji selection beyond the approved 8 states
- add complex facial rigging that distorts the original emoji face

## Rendering Strategy

### Preferred Model

Use fixed emoji image resources for the approved states and render them as textures/sprites inside the existing window.

This gives the best control over:

- stable visual appearance
- predictable sizing
- smooth animation layering
- cross-machine consistency

### Fallback Model

If fixed image sourcing is blocked, a temporary fallback can use native system glyph rendering in preview/prototype flows only.

That fallback is not the preferred runtime design because system emoji rendering can vary by machine and installed font stack.

## Resource Strategy

Each state should have one canonical visual asset.

Expected resource set:

- `happy`
- `sad`
- `angry`
- `sleepy`
- `thinking`
- `hot`
- `mindblown`
- `goodnight`

The current `image` dependency in [Cargo.toml](D:/Project/Deskemoji/Cargo.toml) is only justified if we actually load these assets at runtime. Under this design, that dependency should be used for real asset decoding. If runtime asset loading is deferred, the dependency should be removed until the feature lands.

## Animation Principles

Animation must happen around the emoji, not by redrawing the face.

Allowed motion patterns:

- idle float: very small vertical drift
- idle breathe: very small scale modulation
- tap feedback: brief squash and rebound
- state transition: fade/scale/translate blend between old and new emoji
- hover polish: subtle highlight or emphasis without changing face geometry

Disallowed patterns:

- custom eye tracking that repaints the eyes
- hand-drawn blinking over the emoji face unless it can be done without damaging the approved look
- per-state custom facial rigs that reinterpret the emoji

## State Mapping

Current monitor-driven state logic can remain conceptually the same.

The visual layer changes from:

- state -> custom draw function

to:

- state -> emoji asset
- emoji asset + animation state -> final composed frame

## Architectural Changes

### Current Problem

The current renderer mixes:

- face construction
- state-specific facial drawing
- animation behavior
- low-level pixel plotting

This made iteration expensive and still failed to match the required emoji style.

### Proposed Shape

Split responsibilities into smaller units:

1. **State selection**
   Keeps the existing system-status-to-emotion logic.

2. **Emoji asset catalog**
   Maps each app state to a decoded image resource.

3. **Animation state**
   Tracks idle, tap, hover, and transition timing independently of emoji content.

4. **Compositor**
   Draws the selected emoji asset into the window with simple transforms and opacity.

## File Direction

Expected impacted files:

- [src/main.rs](D:/Project/Deskemoji/src/main.rs)
  - keep app/event loop ownership
  - simplify animation inputs to resource-friendly transforms

- [src/renderer.rs](D:/Project/Deskemoji/src/renderer.rs)
  - stop being the place where facial features are procedurally authored
  - become an image/sprite compositor

- [Cargo.toml](D:/Project/Deskemoji/Cargo.toml)
  - either truly use `image` for resource decoding or remove it until used

- `assets/emoji/*`
  - hold the approved runtime emoji resources

Possible new module:

- `src/emoji_assets.rs`
  - load and expose emoji textures/decoded buffers

## Testing and Verification Expectations

Verification should focus on:

1. resources load correctly
2. every state resolves to the expected emoji asset
3. state transitions do not crash or flicker
4. tap/idle animations preserve image alignment and alpha edges
5. build remains healthy after asset loading is wired in

## Risks

### Asset Fidelity Risk

If the chosen asset source differs from the approved preview style, the app can still feel wrong even after abandoning hand-drawn rendering.

Mitigation:

- verify the exact runtime asset source against the approved preview before fully wiring it in

### Build Pressure Risk

The repository already hit disk-space pressure during `cargo check`.

Mitigation:

- do not keep `image` or PNG decoding code unused
- keep asset-loading scope narrow
- verify whether large generated/unpacked asset folders should remain outside the repo working set

### Animation Overreach Risk

Even with the correct emoji source, excessive transforms can make the result feel fake or toy-like.

Mitigation:

- keep amplitudes small
- prefer easing and short transitions over exaggerated motion

## Success Criteria

This design is successful when:

1. the rendered Deskemoji states look materially the same as the approved emoji references
2. the app no longer depends on hand-drawn facial reconstruction for the main 8 states
3. idle/tap/transition motion feels smooth without altering the emoji identity
4. the asset-loading dependency story in [Cargo.toml](D:/Project/Deskemoji/Cargo.toml) is internally consistent

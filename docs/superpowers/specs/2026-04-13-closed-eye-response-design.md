# Deskemoji Closed-Eye Response Design

**Goal:** Give `sleepy` and `goodnight` a mouse-responsive interaction pass without adding fake eyes or breaking the approved emoji look.

**Context:** Open-eye states already use directional gaze variants. Closed-eye states do not have pupils to move, so they need a different response language that still feels alive.

## Approved Direction

Use a subtle whole-face response instead of synthetic eye overlays:

- `sleepy`: slight horizontal lean toward the cursor, tiny vertical bias, and a soft nodding motion when the cursor is actively offset from the widget center
- `goodnight`: even gentler lean and nod, preserving the calm expression
- no extra glow, no pasted-on features, no speech bubble changes for these states

## Rendering Approach

- Extend sprite transforms to support horizontal offset in addition to the existing scale and vertical offset
- Reuse the existing global cursor delta so behavior is consistent with the open-eye states
- Add a small pure-math helper that maps cursor delta and elapsed time to a closed-eye response transform
- Apply the helper only for `EmojiId::Sleepy` and `EmojiId::Goodnight`

## Constraints

- Keep movement small enough to avoid looking like the whole widget is sliding around
- Preserve the existing approved open-eye gaze-follow behavior
- Keep the implementation asset-light: no new fake-eye layers and no mandatory new PNG authoring for this pass

## Testing

- Add math-level tests for deadzone behavior, directional bias, and smaller `goodnight` amplitude
- Keep existing gaze and bubble tests passing

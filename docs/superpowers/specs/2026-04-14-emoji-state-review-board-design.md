# Deskemoji Emoji State Review Board Design

**Date:** 2026-04-14

## Goal

Create a standalone HTML review board that explains and previews every approved Deskemoji state in one place, with interaction fidelity close to the current Rust app so the page can serve as an ongoing design and development reference.

The board is not a throwaway mockup. It should help future iteration by making the current state set, trigger logic, animation behavior, and review feedback legible in a single artifact.

## Approved Direction

Use a single large matrix-style board with all eight emoji states visible at once.

Each state gets its own review card. The page should avoid unnecessary mode switches or tabs so a reviewer can scan horizontally, compare states quickly, and leave targeted notes directly under the relevant card.

## Scope

The review board will cover these approved states from [src/emoji_assets.rs](D:/Project/Deskemoji/src/emoji_assets.rs):

- `Happy`
- `Sad`
- `Angry`
- `Sleepy`
- `Thinking`
- `Hot`
- `Mindblown`
- `Goodnight`

Each card must present:

- the real emoji asset visuals used by the project
- a live preview that approximates the current app motion language
- the state's Chinese label and emoji character
- a concise explanation of when the state appears
- a concise explanation of how it behaves visually
- a dedicated feedback area for reviewer notes

## Layout Decision

### Overall Structure

The page will use a top summary followed by a full card wall.

Top summary content:

- page title and short usage note
- compact rule strip describing auto-mode state priority and major trigger thresholds
- feedback/export controls

Main content:

- one card per state
- all cards visible in a responsive grid on desktop
- cards stack vertically on narrower screens without losing any content

### Why This Structure

The user explicitly preferred the matrix-wall direction over scenario-first layouts because:

1. all states should be visible together
2. unnecessary switching should be avoided
3. the page should support real comparison work, not guided stepping

The trigger logic still matters, but it should be summarized without dominating the page.

## Fidelity Requirements

The final board must feel meaningfully closer to the real product than a quick visual sketch.

That means fidelity should come from the actual project assets and the current behavior rules, not from generic HTML emoji placeholders.

### Real Asset Usage

The board should use the emoji PNG assets in [assets/emoji](D:/Project/Deskemoji/assets/emoji).

Per-state previews should render the same base and directional variant images the app already uses instead of redrawing faces in HTML or replacing them with text emoji.

### Real Motion Approximation

The board should translate the current motion behavior from [src/renderer.rs](D:/Project/Deskemoji/src/renderer.rs) and [src/main.rs](D:/Project/Deskemoji/src/main.rs) into browser-friendly preview behavior.

This includes:

- idle float and gentle scale modulation
- state-specific accent transforms
- transition feel where useful for explanation
- closed-eye whole-face response style for `Sleepy` and `Goodnight`
- bubble-style overlays for `Hot` and `Mindblown`

The board does not need to literally share runtime code with the Rust app, but its visible behavior should be derived from the same rules closely enough that reviewers are evaluating the real interaction language, not an unrelated animation system.

## Gaze Preview Decision

Do not show a static 9-direction sheet inside every card.

Instead, each card should include a single live gaze-follow preview area:

- open-eye states swap among the existing directional assets based on pointer movement near the preview
- idle pointer returns the state to center
- the direction thresholds should follow the same qualitative behavior as `GazeDirection::from_pointer_delta`
- `Sleepy` and `Goodnight` should not fake pupils; they should demonstrate the existing closed-eye response style instead

This keeps the review board closer to actual usage while avoiding oversized repetitive grids.

## Trigger Logic Presentation

The board should explain real trigger logic without turning the page into a simulator-heavy dashboard.

### Global Rule Strip

A compact top section should summarize the current auto-mode order from [src/main.rs](D:/Project/Deskemoji/src/main.rs):

1. `Hot` when CPU exceeds threshold
2. `Mindblown` when memory exceeds threshold
3. `Sleepy` when idle threshold is reached
4. `Goodnight` during late-night hours
5. `Thinking` during daytime work hours
6. `Happy` as the default fallback

It should also note that some states, such as `Sad` and `Angry`, are currently manual-selection review states rather than auto-selected runtime states.

### Per-Card Trigger Copy

Each state card should include:

- whether the state is auto-selected or manual-only in the current app
- the condition that activates it if auto-selected
- any meaningful priority note if another state can override it

This gives reviewers enough context without requiring a separate scenario switcher.

## Feedback Requirements

The board is intended to support real review work, not just passive viewing.

### Inputs

Each state card should include:

- a multi-line note field for targeted feedback
- a compact status choice such as `保留` / `待改` / `问题大`

### Persistence

Feedback should auto-save in browser local storage as the reviewer types or changes status.

Reloading the page should restore saved notes so the board remains useful across sessions.

### Export

Provide page-level export actions for:

- `JSON`
- `Markdown`

Exports should include:

- export timestamp
- state identifier
- Chinese label
- trigger summary
- reviewer status
- reviewer note

This lets the board feed later issue tracking or follow-up implementation work.

## Implementation Shape

The page should live at:

- [docs/review/deskemoji-emoji-state-review-board.html](D:/Project/Deskemoji/docs/review/deskemoji-emoji-state-review-board.html)

Implementation constraints:

- standalone single HTML file
- no build step
- openable directly in a browser
- should reference repo-local assets
- should remain maintainable enough for future updates as states or trigger rules evolve

## Visual Direction

The board should feel intentional and polished, but not overdesigned.

Visual priorities:

- warm desktop-review tone that fits the existing Deskemoji preview work
- clear grouping and readable card hierarchy
- enough visual character to feel presentation-ready
- no generic gray wireframe treatment

The board should look like a real review artifact, but fidelity effort should be spent first on behavior and asset accuracy.

## Non-Goals

This board should not:

- replace the Rust runtime implementation
- become a separate product UI with independent behavior rules
- require a server or bundler to run
- introduce new canonical emoji states
- invent fake eye rigs for closed-eye expressions
- force reviewers through a multi-step wizard or tab flow

## Success Criteria

This design is successful when:

1. all eight approved states can be reviewed on one page without mode switching
2. the preview visuals use the real project emoji assets
3. reviewers can see motion that materially resembles the current app behavior
4. open-eye states demonstrate gaze-following without static 9-cell clutter
5. each state card supports direct, persistent feedback entry
6. the page can export structured review results for follow-up work
7. the artifact is useful both now and later as a development reference

## Notes

Because the repository currently has unrelated in-progress changes, this design doc should be added without disturbing existing work. Any later implementation should follow the same rule: integrate carefully and avoid reverting user changes.

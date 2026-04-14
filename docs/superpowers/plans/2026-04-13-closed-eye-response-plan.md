# Closed-Eye Response Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add subtle mouse-responsive motion for `sleepy` and `goodnight` while preserving the approved emoji look.

**Architecture:** Keep the current sprite renderer and asset pipeline. Add a small transform helper for closed-eye states, extend render math to support horizontal translation, and only apply the new behavior to the two closed-eye emoji states.

**Tech Stack:** Rust, winit, softbuffer, existing Deskemoji renderer math tests

---

### Task 1: Add failing tests for closed-eye response math

**Files:**
- Modify: `D:/Project/Deskemoji/tests/render_math_test.rs`
- Modify: `D:/Project/Deskemoji/src/renderer.rs`

- [ ] **Step 1: Write the failing test**

Add tests covering:
- deadzone returns almost no closed-eye response
- left/right cursor offset produces matching horizontal lean
- `goodnight` uses a gentler amplitude than `sleepy`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test closed_eye -- --nocapture`
Expected: FAIL because the helper does not exist yet.

- [ ] **Step 3: Write minimal implementation**

Implement a small helper in `src/renderer.rs` that returns a subtle transform delta for closed-eye states.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test closed_eye -- --nocapture`
Expected: PASS

### Task 2: Wire closed-eye response into runtime rendering

**Files:**
- Modify: `D:/Project/Deskemoji/src/main.rs`
- Modify: `D:/Project/Deskemoji/src/renderer.rs`

- [ ] **Step 1: Write the failing test**

Add a narrow state-gating test if needed, or rely on math tests plus compile-time wiring.

- [ ] **Step 2: Run targeted test/compile to verify the gap**

Run: `cargo check --bin deskemoji`
Expected: compile gap until the new transform field/helper is wired through.

- [ ] **Step 3: Write minimal implementation**

Extend `EmojiTransform`/sprite math with horizontal offset and apply the new closed-eye transform only for `sleepy` and `goodnight`.

- [ ] **Step 4: Run verification**

Run: `cargo fmt --check`
Run: `cargo check --bin deskemoji`
Run: `cargo test -- --nocapture`
Expected: all PASS

### Task 3: Build and package a runnable app

**Files:**
- Modify: `D:/Project/Deskemoji/dist/`

- [ ] **Step 1: Build release**

Run: `cargo build --release --bin deskemoji`

- [ ] **Step 2: Package a fresh runnable directory**

Copy the latest executable and current `assets/emoji` into a new `dist` folder without overwriting a running package.

- [ ] **Step 3: Smoke test**

Launch the packaged exe for a short run and confirm the process stays alive.

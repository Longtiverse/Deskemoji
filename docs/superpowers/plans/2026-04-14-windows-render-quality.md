# Windows Render Quality Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current fragile emoji minification path with a higher-quality Windows-friendly rendering path and ship a new Deskemoji build with visibly smoother edges.

**Architecture:** Keep the existing app state machine, monitoring, menu, bubble logic, and gaze logic intact. Rework the renderer around cached high-quality bitmap scaling first, verify the visual gain with real captures, then add any Windows DPI/window fixes needed to ensure the scaled bitmap reaches the desktop 1:1 without extra blur or jaggies.

**Tech Stack:** Rust, winit, Windows desktop APIs, image crate, existing Deskemoji tests and release packaging flow.

---

### Task 1: Lock In the Quality Regression

**Files:**
- Modify: `D:\Project\Deskemoji\tests\render_math_test.rs`

- [ ] Add a failing test for a dedicated high-quality resize helper.
- [ ] Run the focused test to verify it fails for the missing helper.
- [ ] Implement the minimal helper signature needed by the test.
- [ ] Re-run the focused test and keep it green.

### Task 2: Replace Runtime Sprite Scaling

**Files:**
- Modify: `D:\Project\Deskemoji\src\renderer.rs`
- Test: `D:\Project\Deskemoji\tests\render_math_test.rs`

- [ ] Introduce a cached high-quality resize path for emoji sprites.
- [ ] Route sprite composition through the resized bitmap cache instead of per-destination-pixel minification.
- [ ] Keep alpha handling premultiplied-safe so transparent edges do not halo.
- [ ] Re-run focused rendering tests.

### Task 3: Verify Window Pixel Path

**Files:**
- Modify: `D:\Project\Deskemoji\src\main.rs`

- [ ] Audit DPI awareness and physical-size window creation behavior.
- [ ] Apply the smallest fix needed if the window can still be system-scaled.
- [ ] Keep right-click, transparency, and always-on-top behavior unchanged.

### Task 4: Build, Package, and Visual Check

**Files:**
- Modify: `D:\Project\Deskemoji\dist\...`
- Output: `D:\Project\Deskemoji\docs\review\...`

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo check --bin deskemoji`.
- [ ] Run `cargo test -- --nocapture`.
- [ ] Run `cargo build --release --bin deskemoji`.
- [ ] Package a new release directory.
- [ ] Launch the new build and confirm the running process path.
- [ ] Capture fresh screenshots and compare against the previous build.

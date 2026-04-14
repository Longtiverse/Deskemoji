# Layered Window Renderer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Deskemoji's final `softbuffer` presentation path with a Windows layered-window per-pixel-alpha path to improve transparent emoji edge quality.

**Architecture:** Keep the current app state machine, emoji assets, gaze variants, bubble layout, and CPU-side composition. Change only the final presentation layer so rendered pixels are uploaded to a top-down DIB section and presented with `UpdateLayeredWindow` using premultiplied BGRA and `AC_SRC_ALPHA`.

**Tech Stack:** Rust, winit, windows crate, Win32 GDI memory DC/DIB section, `UpdateLayeredWindow`, existing tests.

---

### Task 1: Premultiplied BGRA Conversion

**Files:**
- Modify: `D:\Project\Deskemoji\src\renderer.rs`
- Test: `D:\Project\Deskemoji\tests\render_math_test.rs`

- [ ] Add a failing test for converting internal ARGB pixels into premultiplied BGRA bytes.
- [ ] Run the focused test and confirm it fails because the helper is missing.
- [ ] Implement the helper without touching presentation code.
- [ ] Run the focused test and confirm it passes.

### Task 2: Layered Window Presenter

**Files:**
- Modify: `D:\Project\Deskemoji\src\renderer.rs`
- Modify: `D:\Project\Deskemoji\Cargo.toml` if required by Win32 bindings.

- [ ] Replace the `softbuffer` context/surface fields with a Win32 layered presenter.
- [ ] Capture the window `HWND` from winit's raw window handle in `Renderer::new`.
- [ ] Allocate a compatible memory DC and top-down 32-bit DIB section when size changes.
- [ ] Copy premultiplied BGRA bytes into the DIB section.
- [ ] Call `UpdateLayeredWindow` with `AC_SRC_OVER`, `AC_SRC_ALPHA`, and current window position/size.

### Task 3: Window Style Integration

**Files:**
- Modify: `D:\Project\Deskemoji\src\main.rs`

- [ ] Keep the window borderless, transparent, always-on-top, and skip-taskbar.
- [ ] Ensure the native `WS_EX_LAYERED` style is set on the HWND before first render.
- [ ] Keep right-click menu, dragging, hover, and input monitoring behavior unchanged.

### Task 4: Verification and Release

**Files:**
- Output: `D:\Project\Deskemoji\dist\deskemoji-release-20260415-layered`
- Output: `D:\Project\Deskemoji\docs\review\...`

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo check --bin deskemoji`.
- [ ] Run `cargo test -- --nocapture`.
- [ ] Run `cargo build --release --bin deskemoji`.
- [ ] Package and launch the layered build.
- [ ] Confirm the running process path.
- [ ] Capture a fresh screenshot for visual comparison.

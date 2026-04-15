use deskemoji::dialogue::DialogueManager;
use deskemoji::emoji_assets::EmojiId;
use std::thread;
use std::time::Duration;

#[test]
fn dialogue_manager_loads_builtin_presets() {
    let mgr = DialogueManager::load_default();
    assert!(mgr.enabled);
    assert_eq!(mgr.duration_secs, 5);
}

#[test]
fn trigger_by_state_returns_a_line_and_expires() {
    let mut mgr = DialogueManager::with_config(true, 1, 60);
    mgr.trigger_by_state(EmojiId::Happy);
    assert!(mgr.update().is_some());

    thread::sleep(Duration::from_millis(1100));
    assert!(mgr.update().is_none());
}

#[test]
fn trigger_by_click_overrides_current_line() {
    let mut mgr = DialogueManager::with_config(true, 5, 60);
    mgr.trigger_by_state(EmojiId::Happy);
    let first = mgr.update().unwrap().to_string();

    mgr.trigger_by_click(EmojiId::Angry, 1);
    let second = mgr.update().unwrap().to_string();
    assert_ne!(first, second);
}

#[test]
fn disabled_manager_never_shows_lines() {
    let mut mgr = DialogueManager::with_config(false, 5, 60);
    mgr.trigger_by_state(EmojiId::Hot);
    assert!(mgr.update().is_none());
}

#[test]
fn wake_up_angry_line_changes_with_click_count() {
    let mut mgr = DialogueManager::with_config(true, 5, 60);
    mgr.trigger_by_click(EmojiId::Angry, 1);
    let first = mgr.update().unwrap().to_string();

    mgr.trigger_by_click(EmojiId::Angry, 3);
    let second = mgr.update().unwrap().to_string();
    assert_ne!(first, second);
}

#[test]
fn idle_trigger_respects_interval() {
    let mut mgr = DialogueManager::with_config(true, 5, 1);
    // First call: interval has not elapsed yet (0 < 1)
    assert!(!mgr.trigger_idle(EmojiId::Thinking));

    thread::sleep(Duration::from_millis(1100));
    // Interval elapsed, should trigger
    assert!(mgr.trigger_idle(EmojiId::Thinking));

    // Immediate next call: interval not elapsed
    assert!(!mgr.trigger_idle(EmojiId::Thinking));

    thread::sleep(Duration::from_millis(1100));
    // Interval elapsed again, should trigger
    assert!(mgr.trigger_idle(EmojiId::Thinking));
}

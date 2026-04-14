use deskemoji::emoji_assets::{EmojiAssets, EmojiId};
use deskemoji::renderer::{bubble_label_for_usage, BubbleOverlay, GazeDirection};
use std::path::Path;

#[test]
fn choose_gaze_direction_prefers_cardinal_and_diagonal_variants() {
    assert_eq!(
        GazeDirection::from_pointer_delta(20.0, 0.0),
        GazeDirection::Right
    );
    assert_eq!(
        GazeDirection::from_pointer_delta(-80.0, -80.0),
        GazeDirection::UpLeft
    );
    assert_eq!(
        GazeDirection::from_pointer_delta(0.0, -90.0),
        GazeDirection::Up
    );
    assert_eq!(
        GazeDirection::from_pointer_delta(90.0, 0.0),
        GazeDirection::Right
    );
    assert_eq!(
        GazeDirection::from_pointer_delta(0.0, 0.0),
        GazeDirection::Center
    );
}

#[test]
fn open_eye_states_expose_frame_variants() {
    let assets = EmojiAssets::load_from_dir("assets/emoji").unwrap();

    for emoji in [
        EmojiId::Happy,
        EmojiId::Sad,
        EmojiId::Angry,
        EmojiId::Thinking,
        EmojiId::Hot,
        EmojiId::Mindblown,
    ] {
        assert!(assets.get_variant(emoji, GazeDirection::Center).is_some());
        assert!(assets.get_variant(emoji, GazeDirection::Left).is_some());
        assert!(assets.get_variant(emoji, GazeDirection::UpRight).is_some());
    }
}

#[test]
fn happy_variants_shift_far_enough_to_read_at_widget_scale() {
    let asset_dir = Path::new("assets/emoji");
    let center = image::open(asset_dir.join("happy-center.png"))
        .unwrap()
        .into_rgba8();
    let left = image::open(asset_dir.join("happy-left.png"))
        .unwrap()
        .into_rgba8();
    let right = image::open(asset_dir.join("happy-right.png"))
        .unwrap()
        .into_rgba8();

    assert!(count_rgba_differences(&center, &left) > 3_000);
    assert!(count_rgba_differences(&center, &right) > 3_000);
}

#[test]
fn bubble_label_is_compact_ascii() {
    assert_eq!(bubble_label_for_usage("CPU", 99), "CPU 99%");
    assert_eq!(bubble_label_for_usage("MEM", 89), "MEM 89%");
}

#[test]
fn non_hot_states_do_not_emit_resource_bubble() {
    assert!(BubbleOverlay::for_state(EmojiId::Happy, 24, 48, 0).is_none());
    assert!(BubbleOverlay::for_state(EmojiId::Thinking, 42, 58, 0).is_none());
    assert_eq!(
        BubbleOverlay::for_state(EmojiId::Hot, 99, 58, 0)
            .unwrap()
            .label,
        "CPU 99%"
    );
}

fn count_rgba_differences(a: &image::RgbaImage, b: &image::RgbaImage) -> usize {
    assert_eq!(a.dimensions(), b.dimensions());

    a.pixels()
        .zip(b.pixels())
        .filter(|(left, right)| left.0 != right.0)
        .count()
}

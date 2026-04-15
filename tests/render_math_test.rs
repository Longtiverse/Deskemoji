use deskemoji::emoji_assets::{EmojiId, EmojiImage};
use deskemoji::renderer::{
    bubble_visual_scale, compute_bubble_anchor, compute_bubble_rect, compute_closed_eye_response,
    compute_idle_transform, compute_sprite_center, compute_sprite_rect,
    compute_state_accent_transform, convert_argb_to_premultiplied_bgra, resize_image_high_quality,
    sample_bilinear_rgba, sample_resampled_rgba, SpriteRect,
};

#[test]
fn compute_sprite_rect_keeps_emoji_centered_after_scale_and_bounce() {
    let rect = compute_sprite_rect(120, 120, 60.0, 64.0, 1.1, 0.0, -6.0);

    assert_eq!(rect.center_x, 60);
    assert!(rect.top < 64);
    assert!(rect.width > 0);
    assert!(rect.height > 0);
}

#[test]
fn idle_animation_outputs_small_transform_values() {
    let transform = compute_idle_transform(1.25);

    assert!(transform.offset_y.abs() <= 6.0);
    assert!(transform.scale > 0.95);
    assert!(transform.scale < 1.08);
    assert_eq!(transform.alpha, 1.0);
}

#[test]
fn sample_bilinear_rgba_blends_neighbor_pixels() {
    let image = EmojiImage {
        width: 2,
        height: 2,
        pixels: vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ],
    };

    let sample = sample_bilinear_rgba(&image, 0.5, 0.5);

    assert!((sample[0] as i32 - 128).abs() <= 1);
    assert!((sample[1] as i32 - 128).abs() <= 1);
    assert!((sample[2] as i32 - 128).abs() <= 1);
    assert_eq!(sample[3], 255);
}

#[test]
fn sample_bilinear_rgba_preserves_color_on_transparent_edges() {
    let image = EmojiImage {
        width: 2,
        height: 1,
        pixels: vec![255, 0, 0, 255, 0, 0, 0, 0],
    };

    let sample = sample_bilinear_rgba(&image, 0.5, 0.0);

    assert!(sample[0] >= 250);
    assert_eq!(sample[1], 0);
    assert_eq!(sample[2], 0);
    assert!((sample[3] as i32 - 128).abs() <= 1);
}

#[test]
fn resize_image_high_quality_keeps_transparent_edge_color_coverage() {
    let image = EmojiImage {
        width: 8,
        height: 1,
        pixels: vec![
            255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ],
    };

    let resized = resize_image_high_quality(&image, 1, 1);

    assert_eq!(resized.width, 1);
    assert_eq!(resized.height, 1);
    assert!(
        resized.pixels[0] >= 24,
        "expected red coverage to survive resize, got {:?}",
        resized.pixels
    );
    assert_eq!(resized.pixels[1], 0);
    assert_eq!(resized.pixels[2], 0);
    assert!(
        resized.pixels[3] >= 24,
        "expected alpha coverage to survive resize, got {:?}",
        resized.pixels
    );
}

#[test]
fn convert_argb_to_premultiplied_bgra_matches_layered_window_format() {
    let pixels = [0x80402010, 0xFFFF8040, 0x00000000];
    let converted = convert_argb_to_premultiplied_bgra(&pixels);

    assert_eq!(
        converted,
        vec![8, 16, 32, 128, 64, 128, 255, 255, 0, 0, 0, 0,]
    );
}

#[test]
fn single_pixel_detail_survives_heavy_minification() {
    let image = EmojiImage {
        width: 8,
        height: 1,
        pixels: vec![
            255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ],
    };

    let source_span = image.width as f32;
    let sample = sample_resampled_rgba(&image, source_span / 2.0 - 0.5, 0.0, source_span, 1.0);

    assert!(
        sample[0] >= 24,
        "expected minified sample to retain some red coverage, got {:?}",
        sample
    );
    assert_eq!(sample[1], 0);
    assert_eq!(sample[2], 0);
    assert!(
        sample[3] >= 24,
        "expected minified sample alpha to keep edge coverage, got {:?}",
        sample
    );
}

#[test]
fn compute_bubble_rect_places_bubble_up_left_of_sprite_anchor() {
    let bubble = compute_bubble_rect(120, 120, 96, 84, 72, 34);

    assert!(bubble.left < 96);
    assert!(bubble.top < 84);
    assert!(bubble.tail_tip_x > bubble.left);
    assert!(bubble.tail_tip_y > bubble.top);
}

#[test]
fn bubble_anchor_keeps_status_above_the_sprite_face() {
    let sprite = SpriteRect {
        left: 40,
        top: 75,
        width: 140,
        height: 140,
        center_x: 110,
        center_y: 145,
    };
    let anchor = compute_bubble_anchor(sprite);
    let bubble = compute_bubble_rect(220, 220, anchor.0, anchor.1, 82, 22);

    assert_eq!(anchor.0, sprite.center_x);
    assert!((bubble.left + bubble.width as i32 / 2 - sprite.center_x).abs() <= 1);
    assert!(bubble.top + bubble.height as i32 <= sprite.top);
}

#[test]
fn larger_canvas_renders_emoji_about_fifty_percent_bigger() {
    let (center_x, center_y) = compute_sprite_center(220, 220);
    let rect = compute_sprite_rect(220, 220, center_x, center_y, 1.0, 0.0, 0.0);

    assert!((rect.width as i32 - 141).abs() <= 2);
    assert_eq!(rect.center_x, 110);
    assert!(rect.top >= 70);
}

#[test]
fn larger_sprite_uses_larger_bubble_scale() {
    assert!(bubble_visual_scale(94) < bubble_visual_scale(141));
    assert!(bubble_visual_scale(141) >= 1.45);
}

#[test]
fn closed_eye_response_has_a_deadzone_near_center() {
    let transform = compute_closed_eye_response(EmojiId::Sleepy, 10.0, -8.0, 0.5);

    assert!(transform.offset_x.abs() < 0.2);
    assert!(transform.offset_y.abs() < 0.35);
}

#[test]
fn closed_eye_response_leans_toward_cursor_direction() {
    let left = compute_closed_eye_response(EmojiId::Sleepy, -120.0, 0.0, 0.5);
    let right = compute_closed_eye_response(EmojiId::Sleepy, 120.0, 0.0, 0.5);

    assert!(left.offset_x < -1.0);
    assert!(right.offset_x > 1.0);
}

#[test]
fn goodnight_response_is_gentler_than_sleepy() {
    let sleepy = compute_closed_eye_response(EmojiId::Sleepy, 140.0, -120.0, 0.75);
    let goodnight = compute_closed_eye_response(EmojiId::Goodnight, 140.0, -120.0, 0.75);

    assert!(sleepy.offset_x.abs() > goodnight.offset_x.abs());
    assert!(sleepy.offset_y.abs() > goodnight.offset_y.abs());
}

#[test]
fn hot_accent_is_more_animated_than_happy() {
    let happy = compute_state_accent_transform(EmojiId::Happy, 0.8);
    let hot = compute_state_accent_transform(EmojiId::Hot, 0.8);

    assert!(hot.offset_y.abs() > happy.offset_y.abs());
}

#[test]
fn goodnight_accent_is_calmer_than_sleepy() {
    let sleepy = compute_state_accent_transform(EmojiId::Sleepy, 1.2);
    let goodnight = compute_state_accent_transform(EmojiId::Goodnight, 1.2);

    assert!(sleepy.offset_y.abs() > goodnight.offset_y.abs());
    assert!(sleepy.scale <= goodnight.scale + 0.02);
}

#[test]
fn mindblown_accent_includes_a_subtle_recoil() {
    let transform = compute_state_accent_transform(EmojiId::Mindblown, 0.35);

    assert!(transform.offset_y < 0.0);
    assert!(transform.scale < 1.0);
}

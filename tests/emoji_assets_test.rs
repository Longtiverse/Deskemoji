use deskemoji::emoji_assets::EmojiId;
use std::collections::HashSet;

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

    let expected_data = [
        (EmojiId::Happy, '\u{1F642}', "happy", "\u{5F00}\u{5FC3}"),
        (EmojiId::Sad, '\u{1F622}', "sad", "\u{96BE}\u{8FC7}"),
        (EmojiId::Angry, '\u{1F620}', "angry", "\u{751F}\u{6C14}"),
        (EmojiId::Sleepy, '\u{1F634}', "sleepy", "\u{56F0}\u{5026}"),
        (
            EmojiId::Thinking,
            '\u{1F914}',
            "thinking",
            "\u{601D}\u{8003}",
        ),
        (EmojiId::Hot, '\u{1F975}', "hot", "\u{70ED}"),
        (
            EmojiId::Mindblown,
            '\u{1F92F}',
            "mindblown",
            "\u{5D29}\u{6E83}",
        ),
        (
            EmojiId::Goodnight,
            '\u{1F60C}',
            "goodnight",
            "\u{665A}\u{5B89}",
        ),
    ];

    for (emoji_id, expected_char, expected_stem, expected_label) in expected_data {
        assert_eq!(emoji_id.emoji_char(), expected_char);
        assert_eq!(emoji_id.file_stem(), expected_stem);
        assert_eq!(emoji_id.label_zh(), expected_label);
    }

    let unique_stems: HashSet<_> = EmojiId::all()
        .iter()
        .map(|emoji_id| emoji_id.file_stem())
        .collect();
    assert_eq!(unique_stems.len(), EmojiId::all().len());
}

#[test]
fn loads_png_asset_for_each_approved_state() {
    let assets = deskemoji::emoji_assets::EmojiAssets::load_from_dir("assets/emoji").unwrap();

    for id in EmojiId::all() {
        let image = assets.get(id).unwrap();
        assert!(image.width >= 1024);
        assert!(image.height >= 1024);
        assert_eq!(
            image.pixels.len(),
            (image.width * image.height * 4) as usize
        );
    }
}

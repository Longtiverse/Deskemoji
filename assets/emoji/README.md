# Emoji Runtime Assets

`assets/emoji/` contains runtime PNGs. Deskemoji decodes these files through the `image` crate at startup and renders them directly in the desktop widget.

## Required Files

These transparent PNG filenames must stay aligned with `EmojiId::file_stem()`:

| File | Emoji | Unicode | State |
|------|-------|---------|-------|
| `happy.png` | `🙂` | `U+1F642` | 开心 |
| `sad.png` | `😢` | `U+1F622` | 难过 |
| `angry.png` | `😠` | `U+1F620` | 生气 |
| `sleepy.png` | `😴` | `U+1F634` | 困倦 |
| `thinking.png` | `🤔` | `U+1F914` | 思考 |
| `hot.png` | `🥵` | `U+1F975` | 发热 |
| `mindblown.png` | `🤯` | `U+1F92F` | 崩溃 |
| `goodnight.png` | `😌` | `U+1F60C` | 晚安 |

## Runtime Expectations

- Format: `PNG`
- Background: transparent
- Decode target: RGBA pixel buffer
- Suggested size: `128x128` or `256x256`

If you replace an asset, keep the same filename and the app will load the new PNG on next startup.

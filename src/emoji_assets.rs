use crate::renderer::GazeDirection;
use std::array::from_fn;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmojiImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub struct EmojiAssets {
    images: [EmojiImage; 8],
    variants: [[Option<EmojiImage>; 9]; 8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmojiId {
    Happy,
    Sad,
    Angry,
    Sleepy,
    Thinking,
    Hot,
    Mindblown,
    Goodnight,
}

impl EmojiId {
    pub const fn all() -> [EmojiId; 8] {
        [
            EmojiId::Happy,
            EmojiId::Sad,
            EmojiId::Angry,
            EmojiId::Sleepy,
            EmojiId::Thinking,
            EmojiId::Hot,
            EmojiId::Mindblown,
            EmojiId::Goodnight,
        ]
    }

    pub const fn emoji_char(self) -> char {
        match self {
            EmojiId::Happy => '\u{1F642}',
            EmojiId::Sad => '\u{1F622}',
            EmojiId::Angry => '\u{1F620}',
            EmojiId::Sleepy => '\u{1F634}',
            EmojiId::Thinking => '\u{1F914}',
            EmojiId::Hot => '\u{1F975}',
            EmojiId::Mindblown => '\u{1F92F}',
            EmojiId::Goodnight => '\u{1F60C}',
        }
    }

    pub const fn label_zh(self) -> &'static str {
        match self {
            EmojiId::Happy => "\u{5F00}\u{5FC3}",
            EmojiId::Sad => "\u{96BE}\u{8FC7}",
            EmojiId::Angry => "\u{751F}\u{6C14}",
            EmojiId::Sleepy => "\u{56F0}\u{5026}",
            EmojiId::Thinking => "\u{601D}\u{8003}",
            EmojiId::Hot => "\u{70ED}",
            EmojiId::Mindblown => "\u{5D29}\u{6E83}",
            EmojiId::Goodnight => "\u{665A}\u{5B89}",
        }
    }

    pub const fn file_stem(self) -> &'static str {
        match self {
            EmojiId::Happy => "happy",
            EmojiId::Sad => "sad",
            EmojiId::Angry => "angry",
            EmojiId::Sleepy => "sleepy",
            EmojiId::Thinking => "thinking",
            EmojiId::Hot => "hot",
            EmojiId::Mindblown => "mindblown",
            EmojiId::Goodnight => "goodnight",
        }
    }

    pub const fn index(self) -> usize {
        match self {
            EmojiId::Happy => 0,
            EmojiId::Sad => 1,
            EmojiId::Angry => 2,
            EmojiId::Sleepy => 3,
            EmojiId::Thinking => 4,
            EmojiId::Hot => 5,
            EmojiId::Mindblown => 6,
            EmojiId::Goodnight => 7,
        }
    }
}

impl EmojiAssets {
    pub fn load_default() -> Result<Self, String> {
        let mut candidates = Vec::new();

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                candidates.push(exe_dir.join("assets").join("emoji"));
            }
        }

        if let Ok(current_dir) = std::env::current_dir() {
            candidates.push(current_dir.join("assets").join("emoji"));
        }

        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join("emoji"),
        );

        for candidate in candidates {
            if candidate.exists() {
                return Self::load_from_dir(candidate);
            }
        }

        Err("could not find assets/emoji next to the executable, current directory, or workspace root".to_string())
    }

    pub fn load_from_dir(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let images = [
            load_image(path, EmojiId::Happy)?,
            load_image(path, EmojiId::Sad)?,
            load_image(path, EmojiId::Angry)?,
            load_image(path, EmojiId::Sleepy)?,
            load_image(path, EmojiId::Thinking)?,
            load_image(path, EmojiId::Hot)?,
            load_image(path, EmojiId::Mindblown)?,
            load_image(path, EmojiId::Goodnight)?,
        ];
        let variants = from_fn(|emoji_index| {
            let emoji = EmojiId::all()[emoji_index];
            from_fn(|direction_index| {
                let direction = direction_from_index(direction_index);
                load_variant_image(path, emoji, direction).ok()
            })
        });

        Ok(Self { images, variants })
    }

    pub fn get(&self, id: EmojiId) -> Option<&EmojiImage> {
        self.images.get(id.index())
    }

    pub fn has_gaze_variants(&self, id: EmojiId) -> bool {
        self.variants[id.index()]
            .iter()
            .enumerate()
            .any(|(index, image)| index != GazeDirection::Center.index() && image.is_some())
    }

    pub fn get_variant(&self, id: EmojiId, direction: GazeDirection) -> Option<&EmojiImage> {
        self.variants[id.index()][direction.index()]
            .as_ref()
            .or_else(|| self.get(id))
    }
}

fn direction_from_index(index: usize) -> GazeDirection {
    match index {
        0 => GazeDirection::Center,
        1 => GazeDirection::Left,
        2 => GazeDirection::Right,
        3 => GazeDirection::Up,
        4 => GazeDirection::Down,
        5 => GazeDirection::UpLeft,
        6 => GazeDirection::UpRight,
        7 => GazeDirection::DownLeft,
        8 => GazeDirection::DownRight,
        _ => GazeDirection::Center,
    }
}

fn load_image(base_dir: &Path, id: EmojiId) -> Result<EmojiImage, String> {
    let path = base_dir.join(format!("{}.png", id.file_stem()));
    let decoded = image::open(&path)
        .map_err(|err| format!("failed to decode '{}': {err}", path.display()))?
        .into_rgba8();
    let (width, height) = decoded.dimensions();

    if width == 0 || height == 0 {
        return Err(format!("decoded empty image '{}'", path.display()));
    }

    Ok(EmojiImage {
        width,
        height,
        pixels: decoded.into_raw(),
    })
}

fn load_variant_image(
    base_dir: &Path,
    id: EmojiId,
    direction: GazeDirection,
) -> Result<EmojiImage, String> {
    let suffix = match direction {
        GazeDirection::Center => "center",
        GazeDirection::Left => "left",
        GazeDirection::Right => "right",
        GazeDirection::Up => "up",
        GazeDirection::Down => "down",
        GazeDirection::UpLeft => "ul",
        GazeDirection::UpRight => "ur",
        GazeDirection::DownLeft => "dl",
        GazeDirection::DownRight => "dr",
    };
    let path = base_dir.join(format!("{}-{suffix}.png", id.file_stem()));
    let decoded = image::open(&path)
        .map_err(|err| format!("failed to decode '{}': {err}", path.display()))?
        .into_rgba8();
    let (width, height) = decoded.dimensions();

    if width == 0 || height == 0 {
        return Err(format!("decoded empty image '{}'", path.display()));
    }

    Ok(EmojiImage {
        width,
        height,
        pixels: decoded.into_raw(),
    })
}

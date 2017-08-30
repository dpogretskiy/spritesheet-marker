
use super::sprite::geom;
use super::sprite::*;
use super::sprite::texture_packer::*;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum Vertical {
    Top,
    Bottom,
    Center,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum Horizontal {
    Left,
    Right,
    Center,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum SpriteType {
    Object,
    Platform { horizontal: Vec<Horizontal> },
    Ground {
        vertical: Vec<Vertical>,
        horizontal: Vec<Horizontal>,
    },
}

impl SpriteType {
    pub fn empty_ground() -> SpriteType {
        SpriteType::Ground {
            vertical: vec![],
            horizontal: vec![],
        }
    }

    pub fn empty_platform() -> SpriteType {
        SpriteType::Platform { horizontal: vec![] }
    }
}

// impl Serialize for SpriteType {}

// impl Deserialize for SpriteType {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpriteData {
    pub on_screen_frame: geom::Rect,
    pub frame: geom::Rect,
    pub markers: SpriteType,
    pub name: String,
    pub index: usize,
}

impl SpriteData {
    pub fn create(info: &SpriteSheetInfo) -> Vec<SpriteData> {
        let frames = FrameInfo::extract_frames(&info);
        let marked: Vec<SpriteData> = info.frames
            .iter()
            .enumerate()
            .map(|(ix, sd)| {
                let on_image_frame = sd.frame.clone();
                let on_screen_frame = frames[ix].segment;
                SpriteData {
                    on_screen_frame: geom::Rect::from(on_screen_frame),
                    frame: on_image_frame,
                    markers: SpriteType::Ground {
                        horizontal: vec![],
                        vertical: vec![],
                    },
                    name: sd.filename.clone(),
                    index: ix,
                }
            })
            .collect();
        marked
    }
}

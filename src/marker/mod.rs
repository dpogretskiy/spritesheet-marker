
use super::sprite::geom;
use super::sprite::*;
use super::sprite::texture_packer::*;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum Square {
    LT,
    MT,
    RT,
    LM,
    MM,
    RM,
    LB,
    MB,
    RB,
    IBL,
    ILT,

    IBR,
    IRT,
}

#[derive(Clone)]
pub struct SquareIcon {
    pub s: Square,
    pub icon: &'static str,
    pub rotation: f32,
    pub ix: isize,
}

pub const square_icons: [SquareIcon; 13] = [
    SquareIcon {
        s: Square::LT,
        icon: "\u{f106}",
        rotation: -0.785398,
        ix: 7,
    },
    SquareIcon {
        s: Square::MT,
        icon: "\u{f106}",
        rotation: 0.0,
        ix: 8,
    },
    SquareIcon {
        s: Square::RT,
        icon: "\u{f106}",
        rotation: 0.785398,
        ix: 9,
    },
    SquareIcon {
        s: Square::LM,
        icon: "\u{f104}",
        rotation: 0.0,
        ix: 12,
    },
    SquareIcon {
        s: Square::MM,
        icon: "\u{f111}",
        rotation: 0.0,
        ix: 13,
    },
    SquareIcon {
        s: Square::RM,
        icon: "\u{f105}",
        rotation: 0.0,
        ix: 14,
    },
    SquareIcon {
        s: Square::LB,
        icon: "\u{f107}",
        rotation: 0.785398,
        ix: 17,
    },
    SquareIcon {
        s: Square::MB,
        icon: "\u{f107}",
        rotation: 0.0,
        ix: 18,
    },
    SquareIcon {
        s: Square::RB,
        icon: "\u{f107}",
        rotation: -0.785398,
        ix: 19,
    },
    SquareIcon {
        s: Square::IBL,
        icon: "\u{f106}",
        rotation: 0.785398,
        ix: 21,
    },
    SquareIcon {
        s: Square::ILT,
        icon: "\u{f106}",
        rotation: 0.0,
        ix: 22,
    },
    SquareIcon {
        s: Square::IBR,
        icon: "\u{f106}",
        rotation: -0.785398,
        ix: 25,
    },
    SquareIcon {
        s: Square::IRT,
        icon: "\u{f106}",
        rotation: 0.0,
        ix: 24,
    },
];



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
    Ground { square: Vec<Square> },
}

impl SpriteType {
    pub fn empty_ground() -> SpriteType {
        SpriteType::Ground { square: vec![] }
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
                    markers: SpriteType::Ground { square: vec![] },
                    name: sd.filename.clone(),
                    index: ix,
                }
            })
            .collect();
        marked
    }
}

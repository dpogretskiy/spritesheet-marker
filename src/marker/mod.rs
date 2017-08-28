
use super::sprite::geom;
use super::sprite::*;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Vertical {
    Top,
    Bottom,
    Center,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Horizontal {
    Left,
    Right,
    Center,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SpriteType {
    Animation(usize),
    Object,
    Platform {
        horizontal: Vec<Horizontal>
    },
    Ground {
        vertical: Vec<Vertical>,
        horizontal: Vec<Horizontal>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct SpriteData {
    frame: geom::Rect,
    rotated: bool,
    markers: SpriteType,
    name: String,
    id: usize,
}

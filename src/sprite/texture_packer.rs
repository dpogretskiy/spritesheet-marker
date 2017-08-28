use serde_json;

use std::string::String;
use std::fs::File;
use super::geom::*;

#[derive(Deserialize, Debug, Clone)]
pub struct SpriteSheetInfo {
    pub frames: Vec<Sprite>,
    pub meta: SpriteSheetMeta
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Sprite {
    pub filename: String,
    pub frame: Rect,
    pub rotated: bool,
    pub trimmed: bool,
    pub spriteSourceSize: Rect,
    pub sourceSize: Size,
    pub pivot: Point
}

#[derive(Deserialize, Debug, Clone)]
pub struct SpriteSheetMeta {
    pub app: String,
    pub version: String,
    pub image: String,
    pub format: String,
    pub size: Size,
    pub scale: String,
    pub smartupdate: String
}

use std::path::Path;
use ggez::GameResult;
use ggez::GameError;

impl SpriteSheetInfo {
    pub fn load_info<P: AsRef<Path>>(path: P) -> GameResult<SpriteSheetInfo> {
        let file = File::open(path).unwrap();

        serde_json::from_reader(file).map_err(|e| {
            GameError::ResourceLoadError(format!("{}", e))
        })
    }
}

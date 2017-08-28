use serde_json;

use std::string::String;
use std::result::Result;
use std::fs::File;

#[derive(Deserialize, Debug)]
pub struct SpriteSheet {
    pub frames: Vec<Sprite>,
    pub meta: SpriteSheetMeta
}

#[derive(Deserialize, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32
}

#[derive(Deserialize, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

#[derive(Deserialize, Debug)]
pub struct Size {
    pub w: f32,
    pub h: f32
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

impl SpriteSheet {
    pub fn load_info<P: AsRef<Path>>(path: P) -> Result<SpriteSheet, String> {
        // let file = ctx.filesystem.open(path)?;
        let file = File::open(path)?;

        Ok(serde_json::from_reader(file).map_err(|e| {
            format!("{}", e)
        }))
    }
}

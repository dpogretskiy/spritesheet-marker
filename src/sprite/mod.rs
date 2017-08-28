pub mod texture_packer;
pub mod geom;

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use super::sprite::texture_packer::*;
use super::sprite::geom::*;

use ggez;
use ggez::Context;
use ggez::graphics::Image;
use ggez::GameResult;

use image;
use image::ImageFormat;

pub struct Loader;

impl Loader {
    fn load_meta<P: AsRef<Path>>(path: P) -> GameResult<SpriteSheetInfo> {
        SpriteSheetInfo::load_info(path)
    }

    fn load_image<P: AsRef<Path>>(ctx: &mut Context, path: P) -> GameResult<Image> {
        let img = {
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            image::load(reader, ImageFormat::PNG).unwrap().to_rgba()
        };
        let (width, height) = img.dimensions();
        Image::from_rgba8(ctx, width as u16, height as u16, &img)
    }

    pub fn load_sprite_sheet<P: AsRef<Path>>(
        ctx: &mut Context,
        meta: P,
        image: P,
    ) -> GameResult<SpriteSheet> {
        let info = Loader::load_meta(meta)?;
        let frames = FrameInfo::extract_frames(&info);
        let image = Loader::load_image(ctx, image)?;

        let s = SpriteSheet {
            image: Rc::new(image),
            info,
            frames,
        };
        Ok(s)
    }
}

#[derive(Debug, Clone)]
pub struct SpriteSheet {
    pub image: Rc<Image>,
    pub info: SpriteSheetInfo,
    pub frames: Vec<FrameInfo>,
}

#[derive(Debug, Clone)]
pub struct FrameInfo {
    pub segment: ggez::graphics::Rect,
}

impl FrameInfo {
    pub fn extract_frames(info: &SpriteSheetInfo) -> Vec<FrameInfo> {
        let big = &info.meta;
        let Size {
            w: total_w,
            h: total_h,
        } = big.size;

        info.frames
            .iter()
            .map(|f| {
                let Rect {
                    x: fx,
                    y: fy,
                    w: fw,
                    h: fh,
                } = f.frame;

                let src = ggez::graphics::Rect {
                    x: fx / total_w,
                    y: fy / total_h,
                    w: fw / total_w,
                    h: fh / total_h,
                };

                FrameInfo { segment: src }
            })
            .collect()
    }
}

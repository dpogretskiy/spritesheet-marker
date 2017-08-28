extern crate find_folder;
extern crate ggez;
extern crate image;
extern crate nalgebra;
extern crate serde;
extern crate serde_json;

#[macro_use]
#[cfg(windows)]
extern crate native_windows_gui as nwg;

#[cfg(unix)]
extern crate gtk;

#[macro_use]
extern crate serde_derive;

mod sprite;
mod file_navigator;
mod marker;
mod ui;

use std::path::PathBuf;
use std::time::Duration;
use std::rc::Rc;

use file_navigator::navigator::FileNavigator;
use sprite::{geom, Loader};
use marker::SpriteData;
use ui::GroundUi;
use ui::UiState;

use ggez::{event, graphics, timer, Context, GameResult};
use ggez::graphics::*;
use ggez::conf::Conf;
use ggez::event::MouseState;

fn check_ext(p: &PathBuf, ext: &str) -> bool {
    let sr = format!("{}", p.display());
    sr.ends_with(ext)
}

// fn to_lower_case(p: &PathBuf) -> PathBuf {
//     let sr = format!("{}", p.display());
//     let sr = sr.chars()
//         .flat_map(|c| c.to_lowercase())
//         .collect::<String>();
//     PathBuf::from(sr)
// }

fn select_file() -> (PathBuf, PathBuf) {
    let selected = FileNavigator::select_files(&["png", "json"]);
    println!("{:?}", selected);

    let meta = selected.iter().find(|p| check_ext(p, ".json")).unwrap();
    let image = selected.iter().find(|p| check_ext(p, ".png")).unwrap();
    (meta.clone(), image.clone())
}

pub fn main() {
    let (meta, image) = select_file();
    let gt = std::thread::spawn(move || { lets_play(&meta, &image); });
    gt.join().unwrap();
}

fn lets_play(meta: &PathBuf, image: &PathBuf) {
    let c = Conf {
        window_title: String::from("Geopardy v0.1"),
        window_height: 1000,
        window_width: 1600,
        vsync: true,
        resizable: false,
        window_icon: String::from(""),
    };
    let ctx = &mut Context::load_from_conf("game", "ez", c).unwrap();
    let mut state = Game::new(ctx, meta, image).unwrap();
    event::run(ctx, &mut state).unwrap();
}

pub struct Assets {
    font: Font,
}

impl Assets {
    pub fn load(ctx: &mut Context) -> GameResult<Assets> {
        let font = Font::new(ctx, "/DejaVuSerif.ttf", 18)?;
        Ok(Assets { font })
    }
}

pub struct Game {
    pub assets: Assets,
    pub ui: GroundUi,
    pub sprite: sprite::SpriteSheet,
    pub marked: Vec<SpriteData>,
    pub selection: Rect,
    pub scroll: f32,
    pub render: Vec<(Rc<Image>, DrawParam, Rect)>,
    pub hovered: Option<Rect>,
}

impl Game {
    pub fn new(ctx: &mut Context, meta: &PathBuf, image: &PathBuf) -> GameResult<Game> {
        let sprite = Loader::load_sprite_sheet(ctx, meta, image)?;
        let assets = Assets::load(ctx)?;
        let ui = GroundUi::new(ctx, &assets, Point::new(1400.0, 200.0))?;

        Ok(Game {
            ui,
            assets,
            sprite,
            marked: vec![],
            selection: Rect::zero(),
            scroll: 0.0,
            render: vec![],
            hovered: None,
        })
    }

    pub fn hover(&mut self, x: i32, y: i32) {
        let point = Point::new(x as f32, y as f32);

        let dp = self.render.iter().find(|r| { ui::point_within(&point, &r.2)});

        match dp {
            Some(dp) => {
                self.hovered = {
                    Some(dp.2.clone())
                }
            }
            None => {
                self.ui.hover(&point);
                self.hovered = None
            },
        };
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        let sprite = &self.sprite;
        let frames = sprite::FrameInfo::extract_frames(&sprite.info);

        self.render.clear();
        for (ix, frame) in sprite.info.frames.iter().enumerate() {
            let x = ix % 3;
            let y = ix / 3;
            let src = frames[ix].segment;
            let dest = Point {
                x: 200.0 + x as f32 * 400.0,
                y: (200.0 + y as f32 * 400.0 + self.scroll),
            };
            let geom::Size {
                w: orig_w,
                h: orig_h,
            } = frame.sourceSize;
            let max = orig_w.max(orig_h);
            let scale = Point::new(380.0 / max, 380.0 / max);
            let param = DrawParam {
                src,
                dest,
                scale,
                offset: Point::zero(),
                ..Default::default()
            };
            let on_screen_coordinates = Rect {
                x: dest.x,
                y: dest.y,
                w: orig_w * 380.0 / max,
                h: orig_h * 380.0 / max,
            };
            self.render
                .push((self.sprite.image.clone(), param, on_screen_coordinates));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for &(ref img, params, _) in self.render.iter() {
            graphics::draw_ex(ctx, &**img, params.clone())?;
        }

        if let Some(hover) = self.hovered {
            ui::draw_rect_with_outline(ctx, Color::new(0.0, 0.1, 1.0, 1.0), &hover)?;
        }

        self.ui.draw(ctx);

        graphics::present(ctx);
        timer::sleep_until_next_frame(ctx, 120);
        Ok(())
    }

    fn mouse_motion_event(&mut self, state: MouseState, x: i32, y: i32, _xrel: i32, _yrel: i32) {
        if !state.left() && !state.right() {
            self.hover(x, y);
        }
    }

    fn mouse_button_down_event(&mut self, button: event::MouseButton, x: i32, y: i32) {
        if button == event::MouseButton::Left {
            println!("Leftie!: {} {}", x, y);
        }
    }

    fn mouse_button_up_event(&mut self, _button: event::MouseButton, x: i32, y: i32) {}

    fn mouse_wheel_event(&mut self, _x: i32, y: i32) {
        //1 up, -1 down
        let new_scroll = self.scroll + (y as f32 * 30.0);
        if new_scroll < 0.0 {
            self.scroll = new_scroll
        };
        self.hover(0, 0);
    }
}

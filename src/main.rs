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
use ui::*;

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

use marker::*;
pub fn main() {
    let marked = SpriteData {
        on_screen_frame: geom::Rect::new(1.0, 2.0, 3.0, 4.0),
        frame: geom::Rect::new(4.0, 3.0, 2.0, 1.0),
        markers: SpriteType::Ground {
            horizontal: vec![Horizontal::Left, Horizontal::Right, Horizontal::Center],
            vertical: vec![Vertical::Top, Vertical::Bottom, Vertical::Center],
        },
        name: String::from("raspbery"),
        index: 4,
    };
    let serialized = serde_json::to_string(&marked).unwrap();
    println!("marked: {}", serialized);


    let (meta, image) = select_file();
    let gt = std::thread::spawn(move || { lets_play(&meta, &image); });
    gt.join().unwrap();
}

fn lets_play(meta: &PathBuf, image: &PathBuf) {
    let c = Conf {
        window_title: String::from("Jeopardy v0.1"),
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
    pub ui: AssetTypeUi,
    pub marked: Vec<SpriteData>,
    pub sprites_render: Vec<(DrawParam, usize, Rect)>,
    pub scroll: f32,
    pub image: Rc<Image>,
    pub selected: Option<(Rect, usize)>,
    pub hovered: Option<(Rect, usize)>,
}

impl Game {
    pub fn new(ctx: &mut Context, meta: &PathBuf, image: &PathBuf) -> GameResult<Game> {
        let sprite = Loader::load_sprite_sheet(ctx, meta, image)?;
        let assets = Assets::load(ctx)?;
        let ui = AssetTypeUi::new(ctx, &assets, Point::new(1400.0, 200.0), None)?;

        let marked = SpriteData::create(&sprite.info);
        let image = sprite.image.clone();

        Ok(Game {
            ui,
            assets,
            marked,
            sprites_render: vec![],
            scroll: 0.0,
            image,
            selected: None,
            hovered: None,
        })
    }

    pub fn hover(&mut self, x: i32, y: i32) {
        let point = Point::new(x as f32, y as f32);

        let dp = self.sprites_render
            .iter()
            .find(|tuple| ui::point_within(&point, &tuple.2));

        match dp {
            Some(&(dp, ix, rect)) =>  { 
                let mut r = rect.clone();
                r.y -= self.scroll;
                self.hovered = Some((r, ix)) 
            },
            None => {
                self.ui.hover(&point);
                self.hovered = None
            }
        };
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        self.sprites_render.clear();
        for frame in self.marked.iter() {
            let ix = frame.index;

            let x = ix % 3;
            let y = ix / 3;
            let src = Rect::from(frame.on_screen_frame.clone());
            let dest = Point {
                x: 200.0 + x as f32 * 400.0,
                y: (200.0 + y as f32 * 400.0 + self.scroll),
            };
            let geom::Rect {
                x: _,
                y: _,
                w: orig_w,
                h: orig_h,
            } = frame.frame;
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
            self.sprites_render.push((param, ix, on_screen_coordinates));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for &(params, ix, rect) in self.sprites_render.iter() {
            graphics::draw_ex(ctx, &*self.image, params.clone())?;
        }

        if let Some(hover) = self.hovered {
            ui::draw_rect_with_outline(ctx, Color::new(0.0, 0.1, 1.0, 1.0), 
                &rect_with_scroll(&hover.0, self.scroll))?;
        }

        if let Some(selection) = self.selected {
            ui::draw_rect_with_outline(ctx, Color::new(1.0, 1.0, 1.0, 1.0), 
                &rect_with_scroll(&selection.0, self.scroll))?;
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

            if let Some(sel) = self.selected {
                if ui::point_within(&Point::new(x as f32, y as f32), &rect_with_scroll(&sel.0, self.scroll)) {
                    self.selected = None;
                }
            } else if self.hovered.is_some() {
                self.selected = self.hovered.clone()
            };
        }
    }

    fn mouse_button_up_event(&mut self, _button: event::MouseButton, x: i32, y: i32) {}

    fn mouse_wheel_event(&mut self, _x: i32, y: i32) {
        //1 up, -1 down
        let new_scroll = self.scroll + (y as f32 * 30.0);
        if new_scroll < 0.0 {
            self.scroll = new_scroll
        };
    }
}

fn rect_with_scroll(rect: &Rect, scroll: f32) -> Rect {
    let mut r = rect.clone();
    r.y += scroll;
    r
}

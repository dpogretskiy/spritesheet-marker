extern crate ggez;
extern crate image;
extern crate serde_json;
extern crate serde;

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
use std::fs::File;

use file_navigator::navigator::FileNavigator;
use sprite::{geom, Loader};
use marker::SpriteData;
use ui::*;

use ggez::{event, graphics, timer, Context, GameResult};
use ggez::graphics::*;
use ggez::conf::Conf;
use ggez::event::MouseState;

fn marked_path(meta_path: &PathBuf) -> PathBuf {
    let mut sp = meta_path.clone();
    let name = String::from(sp.file_name().unwrap().to_string_lossy());
    let mut split = name.split('.');

    let ext = split.next_back().unwrap().clone();
    let name: String = split.fold(String::new(), |mut a, s| {
        a.push_str(s);
        a
    });
    let new_name: String = format!("{}-marked.{}", name, ext);
    sp.set_file_name(new_name);

    sp
}

fn check_ext(p: &PathBuf, ext: &str) -> bool {
    let sr = format!("{}", p.display());
    sr.ends_with(ext)
}

fn select_file() -> (PathBuf, PathBuf) {
    let selected = FileNavigator::select_files();

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
        window_title: String::from("Jeopardy v0.1"),
        window_height: 1000,
        window_width: 1600,
        vsync: true,
        resizable: false,
        window_icon: String::from(""),
    };
    let ctx = &mut Context::load_from_conf("game", "ez", c).unwrap();
    let mut state = Game::new(ctx, meta.clone(), image.clone()).unwrap();
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
    marked_path: PathBuf,
    pub assets: Rc<Assets>,
    pub ui: AssetTypeUi,
    pub marked: Vec<SpriteData>,
    pub sprites_render: Vec<(DrawParam, usize, Rect)>,
    pub scroll: f32,
    pub image: Rc<Image>,
    pub selected: Option<(Rect, usize)>,
    pub hovered: Option<(Rect, usize)>,
    pub click: Option<Point>,
}

impl Game {
    pub fn new(ctx: &mut Context, meta_path: PathBuf, image_path: PathBuf) -> GameResult<Game> {
        let sprite = Loader::load_sprite_sheet(ctx, &meta_path, &image_path)?;
        let assets = Rc::new(Assets::load(ctx)?);
        let ui = AssetTypeUi::new(ctx, assets.clone(), Point::new(1400.0, 200.0), None)?;

        let image = sprite.image.clone();

        let marked_path = marked_path(&meta_path);
        let marked: Vec<SpriteData> = if let Some(data) =
            File::open(marked_path.clone()).ok().and_then(|f| {
                serde_json::from_reader(f).ok()
            })
        {
            data
        } else {
            SpriteData::create(&sprite.info)
        };

        Ok(Game {
            marked_path,
            ui,
            assets,
            marked,
            sprites_render: vec![],
            scroll: 0.0,
            image,
            selected: None,
            hovered: None,
            click: None,
        })
    }

    pub fn hover(&mut self, x: i32, y: i32) {
        let point = Point::new(x as f32, y as f32);

        let dp = self.sprites_render.iter().find(|tuple| {
            ui::point_within(&point, &tuple.2)
        });

        match dp {
            Some(&(_, ix, rect)) => {
                let mut r = rect.clone();
                r.y -= self.scroll;
                self.hovered = Some((r, ix))
            }
            None => {
                self.ui.hover(&point);
                self.hovered = None
            }
        };
    }

    pub fn unselect(&mut self) {
        if let Some((_, ix)) = self.selected {
            self.marked[ix] = self.ui.full_state().unwrap();
            self.selected = None;
        } else {
            panic!("Nothing is selected!")
        };
    }

    pub fn select(&mut self, ix: usize, ctx: &mut Context) {
        if let None = self.selected {
            let ui = AssetTypeUi::new(
                ctx,
                self.assets.clone(),
                Point::new(1400.0, 200.0),
                Some(&self.marked[ix]),
            ).unwrap();
            self.selected = self.hovered.clone();
            self.ui = ui;
        } else {
            panic!("Selected already!")
        };
    }

    pub fn save(&mut self) {
        if self.selected.is_some() {
            self.unselect();
        }

        let sp = &self.marked_path;

        let file = File::create(sp).unwrap();
        serde_json::to_writer_pretty(file, &self.marked).unwrap();
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        let mut save_now = false;

        if let Some(ref point) = self.click.map(|c| c.clone()) {
            if let Some(sel) = self.selected {
                if ui::point_within(&point, &rect_with_scroll(&sel.0, self.scroll)) {
                    self.unselect();
                } else if let Some(hovered) = self.hovered {
                    self.unselect();
                    self.select(hovered.1, ctx);
                } else {
                    let opt = self.ui.interact(ctx, point).err();
                    if let Some(ggez::GameError::UnknownError(_)) = opt {
                        save_now = true;
                    };
                };
            } else if let Some(hovered) = self.hovered {
                self.select(hovered.1, ctx);
            };
        };
        self.click = None;

        if save_now {
            self.save();
        };

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

        for &(params, _, _) in self.sprites_render.iter() {
            graphics::draw_ex(ctx, &*self.image, params.clone())?;
        }

        if let Some(hover) = self.hovered {
            ui::draw_rect_with_outline(
                ctx,
                Color::new(0.0, 0.1, 1.0, 1.0),
                &rect_with_scroll(&hover.0, self.scroll),
            )?;
        }

        if let Some(selection) = self.selected {
            ui::draw_rect_with_outline(
                ctx,
                Color::new(1.0, 1.0, 1.0, 1.0),
                &rect_with_scroll(&selection.0, self.scroll),
            )?;
        }

        if self.selected.is_some() {
            self.ui.draw(ctx);
        };

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
            self.click = Some(Point::new(x as f32, y as f32));
        }
    }

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

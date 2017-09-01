
use ggez::graphics::*;
use ggez::graphics;
use ggez::{Context, GameResult};
use super::Assets;
use super::marker::*;
use std::collections::HashSet;
use std::cmp::PartialEq;
use std::hash::Hash;
use std::rc::Rc;
use std::borrow::Borrow;
use ggez::GameError;
use serde_json;
use std::collections::HashMap;

const H_SPACE: f32 = 120.0;
const W_SPACE: f32 = 50.0;

const W_SIZE: f32 = 115.0;
const H_SIZE: f32 = 45.0;

type SBST = SimpleButton<SpriteType>;

pub struct SimpleButton<T> {
    pub rect: Rect,
    pub text: Text,
    pub offset: Point,
    pub action: Box<(FnMut(&mut T) -> ())>,
}

impl<T> PartialEq for SimpleButton<T> {
    fn eq(&self, other: &SimpleButton<T>) -> bool {
        self.rect == other.rect && self.text.contents() == other.text.contents()
    }
}

impl<T> SimpleButton<T> {
    pub fn new33(
        ctx: &mut Context,
        assets: Rc<Assets>,
        text: &str,
        position: usize,
        offset: &Point,
        action: Box<FnMut(&mut T) -> ()>,
    ) -> SimpleButton<T> {
        let v = (position - 1) / 3;
        let h = (position - 1) % 3;

        let text = Text::new(ctx, text, &assets.font).unwrap();
        let rect = Rect::new(
            H_SPACE * (h - 1) as f32 + offset.x,
            W_SPACE * (v - 1) as f32 + offset.y,
            W_SIZE,
            H_SIZE,
        );

        SimpleButton::<T> {
            rect,
            text,
            offset: offset.clone(),
            action: action,
        }
    }

    pub fn interact(&self, point: &Point, val: &mut T) -> bool {
        let yes = point_within(point, &self.rect);
        if yes {
            self.action.as_ref()(val)
        };
        yes
    }

    pub fn hover(&self, point: &Point) -> bool {
        point_within(point, &self.rect)
    }

    pub fn draw(&self, ctx: &mut Context) {
        let center = center(&self.rect);
        graphics::rectangle(ctx, DrawMode::Line, self.rect.clone()).unwrap();
        graphics::draw(ctx, &self.text, center, 0.0).unwrap();
    }
}

pub trait UiState {
    fn draw(&self, ctx: &mut Context);
    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()>;
    fn hover(&mut self, point: &Point) -> Option<Rect>;
    fn return_state(&self) -> Option<SpriteType>;
}

type SBAT = SimpleButton<(AssetTypeUi)>;

pub struct AssetTypeUi {
    assets: Rc<Assets>,
    offset: Point,
    sub_ui: Box<UiState>,
    state: Option<SpriteData>,
    selected: Option<Rect>,
    hovered: Option<Rect>,
    object: SBAT,
    platform: SBAT,
    ground: SBAT,
    save: SBAT,
}

impl AssetTypeUi {
    pub fn new(
        ctx: &mut Context,
        assets: Rc<Assets>,
        offset: Point,
        data: Option<&SpriteData>,
    ) -> GameResult<AssetTypeUi> {
        let ui = AssetTypeUi::build_sub_ui(data.map(|d| &d.markers), &offset, ctx, assets)?;
        let op = &offset;

        let object: SBAT = SimpleButton::new33(
            ctx,
            assets,
            "Object",
            4,
            op,
            Box::new(|tuple: (&mut AssetTypeUi, &mut Context)| {
                let atui = tuple.0;
                let ctx = tuple.1;


                let ui = AssetTypeUi::build_sub_ui(
                    Some(&SpriteType::Object),
                    &atui.offset,
                    ctx,
                    atui.assets,
                ).unwrap();
                atui.selected = Some(atui.object.rect.clone());
                atui.sub_ui = ui;
            }),
        );

        let platform: SBAT = SimpleButton::new33(
            ctx,
            assets,
            "Platform",
            5,
            op,
            Box::new(|atui: &mut AssetTypeUi| {
                let ui = AssetTypeUi::build_sub_ui(
                    Some(&SpriteType::empty_ground()),
                    &atui.offset,
                    ctx,
                    atui.assets,
                ).unwrap();
                atui.selected = Some(atui.ground.rect.clone());
                atui.sub_ui = ui;
            }),
        );

        let ground: SBAT = SimpleButton::new33(
            ctx,
            assets,
            "Ground",
            6,
            op,
            Box::new(|atui: &mut AssetTypeUi| {
                let ui = AssetTypeUi::build_sub_ui(
                    Some(&SpriteType::empty_platform()),
                    &atui.offset,
                    ctx,
                    atui.assets,
                ).unwrap();
                atui.selected = Some(atui.platform.rect.clone());
                atui.sub_ui = ui;
            }),
        );

        let save: SBAT = SimpleButton::new33(
            ctx,
            assets,
            "SAVE!",
            5,
            &Point::new(1450.0, 850.0),
            Box::new(|mut x: &mut AssetTypeUi| ()),
        );

        let selected = match ui.return_state() {
            Some(SpriteType::Ground { .. }) => Some(ground.rect.clone()),
            Some(SpriteType::Platform { .. }) => Some(platform.rect.clone()),
            Some(SpriteType::Object) => Some(object.rect.clone()),
            None => None,
        };

        Ok(AssetTypeUi {
            assets,
            offset,
            sub_ui: ui,
            selected,
            state: data.map(|d| d.clone()),
            hovered: None,
            object,
            platform,
            ground,
            save,
        })
    }

    fn build_sub_ui(
        data: Option<&SpriteType>,
        offset: &Point,
        ctx: &mut Context,
        assets: Rc<Assets>,
    ) -> GameResult<Box<UiState>> {
        match data.map(|d| d.clone()) {
            Some(markers) => {
                let mut sub_ui_offset = offset.clone();
                sub_ui_offset.y += 300.0;

                let ui: Box<UiState> = match markers {
                    SpriteType::Ground { square: sqr } => {
                        let ground = GroundUi::new(ctx, assets, sub_ui_offset, sqr)?;
                        Box::new(ground)
                    }
                    SpriteType::Platform { horizontal: hor } => {
                        let platform = PlatformUi::new(ctx, assets, sub_ui_offset, hor)?;
                        Box::new(platform)
                    }
                    SpriteType::Object => Box::new(NoSubUi {
                        state: Some(SpriteType::Object),
                    }),
                };
                Ok(ui)
            }
            None => Ok(Box::new(NoSubUi { state: None })),
        }
    }

    pub fn full_state(&self) -> Option<SpriteData> {
        let state = self.state.clone();

        state.and_then(|mut state| {
            self.return_state().map(|s| {
                state.markers = s.clone();
                state
            })
        })
    }
}

impl UiState for AssetTypeUi {
    fn draw(&self, ctx: &mut Context) {
        self.object.draw(ctx);
        self.platform.draw(ctx);
        self.ground.draw(ctx);
        self.save.draw(ctx);

        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h).unwrap();
        };

        if let Some(s) = self.selected {
            draw_rect_with_outline(ctx, Color::new(0.0, 0.8, 0.2, 1.0), &s).unwrap();
        }

        self.sub_ui.draw(ctx);
    }

    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()> {
        if (self.ground.interact(point, self)) {
        } else if (self.platform.interact(point, self)) {
        } else if (self.object.interact(point, self)) {
        } else if (self.save.interact(point, self)) {
            return Err(GameError::from(String::from("Save now")));
        } else {
            self.sub_ui.interact(ctx, point)?;
        };
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let rect: Option<Rect> = vec![&self.object, &self.platform, &self.ground, &self.save]
            .iter()
            .find(|b| b.hover(point))
            .map(|b| b.rect.clone());
        self.hovered = rect;
        if let None = rect {
            self.sub_ui.hover(point);
        };
        rect
    }

    fn return_state(&self) -> Option<SpriteType> {
        self.sub_ui.return_state()
    }
}

pub struct GroundUi {
    offset: Point,
    hovered: Option<Rect>,
    state: Vec<Square>,
    all: Vec<Square>,
    buttons: Vec<SBGU>,
}

type SBGU = SimpleButton<Vec<Square>>;
impl GroundUi {
    pub fn new(
        ctx: &mut Context,
        assets: Rc<Assets>,
        offset: Point,
        state: Vec<Square>,
    ) -> GameResult<GroundUi> {
        let op = &offset;

        let all = vec![
            Square::LT,
            Square::MT,
            Square::RT,
            Square::LM,
            Square::MM,
            Square::RM,
            Square::LB,
            Square::MB,
            Square::RB,
        ];

        let buttons: Vec<SBGU> = all.iter()
            .enumerate()
            .map(|(ix, s)| {
                SimpleButton::new33(
                    ctx,
                    assets,
                    serde_json::to_string(s).unwrap().as_str(),
                    ix + 1,
                    &offset,
                    Box::new(|state: &mut Vec<Square>| {
                        if !state.contains(&s) {
                            distinct_vec_add(&mut state, *s);
                        } else {
                            state.retain(|i| *i != *s);
                        };
                    }),
                )
            })
            .collect();

        Ok(GroundUi {
            offset,
            buttons,
            all,
            hovered: None,
            state,
        })
    }

    fn draw_selected(&self, ctx: &mut Context) {
        let color = Color::new(0.0, 0.8, 0.2, 1.0);

        let all_ix = self.all
            .iter()
            .enumerate()
            .map(|(s, ix)| (ix, s))
            .collect::<HashMap<_, _>>();

        for s in self.state.iter() {
            let ix = all_ix.get(s).unwrap();
            let rect = self.buttons[*ix].rect.clone();
            draw_rect_with_outline(ctx, color.clone(), &rect).unwrap();
        }
    }
}

impl UiState for GroundUi {
    fn draw(&self, ctx: &mut Context) {
        for b in self.buttons.iter() {
            b.draw(ctx);
        }

        self.draw_selected(ctx);

        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h).unwrap();
        };
    }

    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()> {
        let ns = self.state.clone();
        let bo = self.buttons.iter().find(|b| b.interact(point, &mut ns));
        self.state = ns;
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let rect = self.buttons
            .iter()
            .find(|b| b.hover(point))
            .map(|b| b.rect.clone());
        self.hovered = rect;
        rect
    }

    fn return_state(&self) -> Option<SpriteType> {
        Some(SpriteType::Ground {
            square: self.state.clone(),
        })
    }
}

type SBPI = SimpleButton<Vec<Horizontal>>;
pub struct PlatformUi {
    offset: Point,
    hovered: Option<Rect>,
    state: Vec<Horizontal>,
    left: SBPI,
    center: SBPI,
    right: SBPI,
}

impl PlatformUi {
    pub fn new(
        ctx: &mut Context,
        assets: Rc<Assets>,
        offset: Point,
        state: Vec<Horizontal>,
    ) -> GameResult<PlatformUi> {
        let clj = |state: &mut Vec<Horizontal>, s: Horizontal| {
            if !state.contains(&s) {
                distinct_vec_add(&mut state, s);
            } else {
                state.retain(|i| *i != s);
            };
        };

        let left = SimpleButton::new33(
            ctx,
            assets,
            "Left",
            4,
            &offset,
            Box::new(|state: &mut Vec<Horizontal>| clj(state, Horizontal::Left)),
        );
        let center = SimpleButton::new33(
            ctx,
            assets,
            "Center",
            5,
            &offset,
            Box::new(|state: &mut Vec<Horizontal>| clj(state, Horizontal::Center)),
        );
        let right = SimpleButton::new33(
            ctx,
            assets,
            "Right",
            6,
            &offset,
            Box::new(|state: &mut Vec<Horizontal>| clj(state, Horizontal::Right)),
        );

        Ok(PlatformUi {
            offset,
            hovered: None,
            state,
            left,
            center,
            right,
        })
    }

    pub fn to_vec(&self) -> Vec<&SBPI> {
        vec![&self.left, &self.center, &self.right]
    }

    fn draw_selected(&self, ctx: &mut Context) {
        let color = Color::new(0.0, 0.8, 0.2, 1.0);
        let offset = &self.offset;

        if self.state.contains(&Horizontal::Left) {
            draw_rect_with_outline(ctx, color.clone(), &self.left.rect).unwrap();
        };
        if self.state.contains(&Horizontal::Right) {
            draw_rect_with_outline(ctx, color.clone(), &self.right.rect).unwrap();
        };
        if self.state.contains(&Horizontal::Center) {
            draw_rect_with_outline(ctx, color.clone(), &self.center.rect).unwrap();
        };
    }
}

impl UiState for PlatformUi {
    fn draw(&self, ctx: &mut Context) {
        for b in self.to_vec().iter() {
            b.draw(ctx);
        }

        self.draw_selected(ctx);

        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h).unwrap();
        };
    }

    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()> {
        self.to_vec()
            .iter()
            .find(|b| b.interact(point, &mut self.state));
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let rect = self.to_vec()
            .iter()
            .find(|b| b.hover(point))
            .map(|b| b.rect.clone());
        self.hovered = rect;
        rect
    }

    fn return_state(&self) -> Option<SpriteType> {
        Some(SpriteType::Platform {
            horizontal: self.state.clone(),
        })
    }
}

pub struct NoSubUi {
    state: Option<SpriteType>,
}

impl UiState for NoSubUi {
    fn draw(&self, _ctx: &mut Context) {}

    fn interact(&mut self, _ctx: &mut Context, _point: &Point) -> GameResult<()> {
        Ok(())
    }

    fn hover(&mut self, _point: &Point) -> Option<Rect> {
        None
    }

    fn return_state(&self) -> Option<SpriteType> {
        self.state.clone()
    }
}

pub fn point_within(point: &Point, rect: &Rect) -> bool {
    let &Point { x, y } = point;
    rect.bottom() < y && rect.top() > y && rect.left() < x && rect.right() > x
}

pub fn draw_rect_with_outline(ctx: &mut Context, mut color: Color, rect: &Rect) -> GameResult<()> {
    color.a = 0.1;
    graphics::set_color(ctx, color)?;
    graphics::rectangle(ctx, DrawMode::Fill, rect.clone())?;
    color.a = 0.7;
    graphics::set_color(ctx, color)?;
    graphics::rectangle(ctx, DrawMode::Line, rect.clone())?;
    graphics::set_color(ctx, graphics::WHITE)?;
    Ok(())
}

fn distinct_vec_add<T: Hash + Eq + PartialEq>(vec: &mut Vec<T>, value: T) {
    vec.push(value);
    let set: HashSet<_> = vec.drain(..).collect();
    vec.extend(set.into_iter());
}

fn center(r: &Rect) -> Point {
    Point::new((r.left() + r.right()) / 2.0, (r.top() + r.bottom()) / 2.0)
}

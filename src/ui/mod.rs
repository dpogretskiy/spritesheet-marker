
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
use std::cell::RefCell;

const H_SPACE: f32 = 120.0;
const W_SPACE: f32 = 50.0;

const W_SIZE: f32 = 115.0;
const H_SIZE: f32 = 45.0;

type SBST = SimpleButton<SpriteType>;

pub struct SimpleButton<T> {
    pub rect: Rect,
    pub text: Text,
    pub rotation: f32,
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
        let v = (position as isize - 1) / 3;
        let h = (position as isize - 1) % 3;

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
            rotation: 0.0,
            offset: offset.clone(),
            action: action,
        }
    }

    pub fn new55(
        ctx: &mut Context,
        assets: Rc<Assets>,
        text: &str,
        position: isize,
        rotation: f32,
        offset: &Point,
        action: Box<FnMut(&mut T) -> ()>,
    ) -> SimpleButton<T> {
        let v = (position - 1) / 5;
        let h = (position - 1) % 5;

        let space = 65.0;
        let size = 60.0;

        let text = Text::new(ctx, text, &assets.awesome).unwrap();
        let rect = Rect::new(
            space * (h - 2) as f32 + offset.x,
            space * (v - 2) as f32 + offset.y,
            size,
            size,
        );

        SimpleButton::<T> {
            rect,
            text,
            rotation,
            offset: offset.clone(),
            action,
        }
    }

    pub fn interact(&mut self, point: &Point, val: &mut T) -> bool {
        let yes = point_within(point, &self.rect);
        if yes {
            (*self.action)(val)
        };
        yes
    }

    pub fn hover(&self, point: &Point) -> bool {
        point_within(point, &self.rect)
    }

    pub fn draw(&self, ctx: &mut Context) {
        let center = center(&self.rect);
        graphics::rectangle(ctx, DrawMode::Line, self.rect.clone()).unwrap();
        graphics::draw(ctx, &self.text, center, self.rotation).unwrap();
    }
}

pub trait UiState {
    fn draw(&self, ctx: &mut Context);
    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()>;
    fn hover(&mut self, point: &Point) -> Option<Rect>;
    fn return_state(&self) -> Option<SpriteType>;
}

type SBAT = SimpleButton<SubUiContainer>;

type SubUi = Rc<RefCell<Box<UiState>>>;

pub struct AssetUiButtons {
    object: SBAT,
    platform: SBAT,
    ground: SBAT,
    save: SBAT,
}

pub struct SubUiContainer {
    sub_ui: SubUi,
    object_ui: SubUi,
    platform_ui: SubUi,
    ground_ui: SubUi,
}

pub struct AssetTypeUi {
    assets: Rc<Assets>,
    offset: Point,
    state: Option<SpriteData>,
    hovered: Option<Rect>,
    selected: Option<Rect>,
    sub_ui_container: SubUiContainer,
    buttons: AssetUiButtons,
}

impl AssetTypeUi {
    pub fn new(
        ctx: &mut Context,
        assets: Rc<Assets>,
        offset: Point,
        data: Option<&SpriteData>,
    ) -> GameResult<AssetTypeUi> {
        let ui = AssetTypeUi::build_sub_ui(data.map(|d| &d.markers), &offset, ctx, assets.clone())?;
        let op = &offset;

        let mut object_ui = Rc::new(
            AssetTypeUi::build_sub_ui(Some(&SpriteType::Object), op, ctx, assets.clone()).unwrap(),
        );
        let object: SBAT = SimpleButton::new33(
            ctx,
            assets.clone(),
            "Object",
            4,
            op,
            Box::new(|ui: &mut SubUiContainer| {
                ui.sub_ui = ui.object_ui.clone();
            }),
        );

        let mut platform_ui = Rc::new(
            AssetTypeUi::build_sub_ui(Some(&SpriteType::empty_platform()), op, ctx, assets.clone())
                .unwrap(),
        );
        let platform: SBAT = SimpleButton::new33(
            ctx,
            assets.clone(),
            "Platform",
            5,
            op,
            Box::new(|ui: &mut SubUiContainer| {
                ui.sub_ui = ui.platform_ui.clone();
            }),
        );

        let mut ground_ui = Rc::new(
            AssetTypeUi::build_sub_ui(Some(&SpriteType::empty_platform()), op, ctx, assets.clone())
                .unwrap(),
        );
        let ground: SBAT = SimpleButton::new33(
            ctx,
            assets.clone(),
            "Ground",
            6,
            op,
            Box::new(|ui: &mut SubUiContainer| {
                ui.sub_ui = ui.ground_ui.clone();
            }),
        );

        let save: SBAT = SimpleButton::new33(
            ctx,
            assets.clone(),
            "SAVE!",
            5,
            &Point::new(1450.0, 850.0),
            Box::new(|x| ()),
        );

        let final_ui = Rc::new(ui);
        let state = (*(*final_ui).borrow_mut()).return_state();

        let mut selected = None;

        match state {
            Some(SpriteType::Ground { .. }) => {
                ground_ui = final_ui.clone();
                selected = Some(ground.rect.clone());
            }
            Some(SpriteType::Platform { .. }) => {
                platform_ui = final_ui.clone();
                selected = Some(platform.rect.clone());
            }
            Some(SpriteType::Object) => {
                object_ui = final_ui.clone();
                selected = Some(object.rect.clone());
            }
            _ => (),
        };

        Ok(AssetTypeUi {
            assets,
            offset,
            state: data.map(|d| d.clone()),
            hovered: None,
            selected,
            sub_ui_container: SubUiContainer {
                sub_ui: final_ui,
                object_ui: object_ui,
                platform_ui: platform_ui,
                ground_ui: ground_ui,
            },
            buttons: AssetUiButtons {
                object,
                platform,
                ground,
                save,
            },
        })
    }

    fn build_sub_ui(
        data: Option<&SpriteType>,
        offset: &Point,
        ctx: &mut Context,
        assets: Rc<Assets>,
    ) -> GameResult<RefCell<Box<UiState>>> {
        let result: Box<UiState> = match data.map(|d| d.clone()) {
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
                ui
            }
            None => Box::new(NoSubUi { state: None }),
        };
        Ok(RefCell::new(result))
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
        self.buttons.borrow().object.draw(ctx);
        self.buttons.borrow().platform.draw(ctx);
        self.buttons.borrow().ground.draw(ctx);
        self.buttons.borrow().save.draw(ctx);

        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h).unwrap();
        };

        if let Some(sel) = self.selected {
            draw_rect_with_outline(ctx, Color::new(0.0, 0.8, 0.2, 1.0), &sel).unwrap();
        };

        (*self.sub_ui_container.sub_ui).borrow().draw(ctx);
    }

    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()> {
        let buttons = &mut self.buttons;
        let mut ui = &mut self.sub_ui_container;

        if buttons.ground.interact(point, ui) {
            self.selected = Some(buttons.ground.rect.clone());
        } else if buttons.platform.interact(point, ui) {
            self.selected = Some(buttons.platform.rect.clone());
        } else if buttons.object.interact(point, ui) {
            self.selected = Some(buttons.object.rect.clone());
        } else if buttons.save.interact(point, ui) {
            return Err(GameError::from(String::from("Save now")));
        } else {
            (*ui.sub_ui).borrow_mut().interact(ctx, point)?
        };
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let ui = &mut self.sub_ui_container;
        let buttons = self.buttons.borrow();
        let buttons_vec = vec![
            &buttons.object,
            &buttons.platform,
            &buttons.ground,
            &buttons.save,
        ];

        let rect: Option<Rect> = buttons_vec
            .iter()
            .find(|b| b.hover(point))
            .map(|b| b.rect.clone());
        self.hovered = rect;
        if let None = rect {
            (*ui.sub_ui).borrow_mut().hover(point);
        };
        rect
    }

    fn return_state(&self) -> Option<SpriteType> {
        (*self.sub_ui_container.sub_ui).borrow().return_state()
    }
}

pub struct GroundUi {
    offset: Point,
    hovered: Option<Rect>,
    state: Vec<Square>,
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

        let buttons: Vec<SBGU> = square_icons
            .iter()
            .map(|si| {
                let si = si.clone();
                SimpleButton::new55(
                    ctx,
                    assets.clone(),
                    si.icon,
                    si.ix,
                    si.rotation,
                    &offset,
                    Box::new(move |state: &mut Vec<Square>| {
                        if !state.contains(&si.s) {
                            distinct_vec_add(state, si.s.clone());
                        } else {
                            state.retain(|i| *i != si.s.clone());
                        };
                    }),
                )
            })
            .collect();

        Ok(GroundUi {
            offset,
            buttons,
            hovered: None,
            state,
        })
    }

    fn draw_selected(&self, ctx: &mut Context) {
        let color = Color::new(0.0, 0.8, 0.2, 1.0);

        let all_ix = square_icons
            .iter()
            .enumerate()
            .map(|(ix, si)| (si.s.clone(), ix))
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
        let mut ns = self.state.clone();

        'find: for mut b in self.buttons.iter_mut() {
            if b.interact(point, &mut ns) {
                break 'find;
            }
        }
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
        let clj = |mut state: &mut Vec<Horizontal>, s: Horizontal| {
            if !state.contains(&s) {
                distinct_vec_add(&mut state, s);
            } else {
                state.retain(|i| *i != s);
            };
        };

        let clj = Rc::new(clj);
        let clj2 = clj.clone();
        let clj3 = clj.clone();

        let left = SimpleButton::new33(
            ctx,
            assets.clone(),
            "Left",
            4,
            &offset,
            Box::new(move |state: &mut Vec<Horizontal>| {
                clj(state, Horizontal::Left)
            }),
        );
        let center = SimpleButton::new33(
            ctx,
            assets.clone(),
            "Center",
            5,
            &offset,
            Box::new(move |state: &mut Vec<Horizontal>| {
                clj2(state, Horizontal::Center)
            }),
        );
        let right = SimpleButton::new33(
            ctx,
            assets.clone(),
            "Right",
            6,
            &offset,
            Box::new(move |state: &mut Vec<Horizontal>| {
                clj3(state, Horizontal::Right)
            }),
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
        let mut vec = vec![&self.left, &self.center, &self.right];
        for b in vec.iter() {
            b.draw(ctx);
        }

        self.draw_selected(ctx);

        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h).unwrap();
        };
    }

    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()> {
        let mut state = self.state.clone();
        let mut vec = vec![&mut self.left, &mut self.center, &mut self.right];

        'find: for ref mut b in vec.iter_mut() {
            if b.interact(point, &mut state) {
                break 'find;
            }
        }
        self.state = state;
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let mut vec = vec![&mut self.left, &mut self.center, &mut self.right];

        let rect = vec.iter().find(|b| b.hover(point)).map(|b| b.rect.clone());
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


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

const H_SPACE: f32 = 120.0;
const W_SPACE: f32 = 50.0;

const W_SIZE: f32 = 115.0;
const H_SIZE: f32 = 45.0;

pub struct SimpleButton {
    pub rect: Rect,
    pub text: Text,
}

impl PartialEq for SimpleButton {
    fn eq(&self, other: &SimpleButton) -> bool {
        self.rect == other.rect && self.text.contents() == other.text.contents()
    }
}

impl SimpleButton {
    pub fn new(text: Text, rect: Rect) -> SimpleButton {
        SimpleButton { rect, text }
    }

    pub fn point_within(&self, point: &Point, offset: &Point) -> bool {
        let mut c_rect = self.rect.clone();
        c_rect.x += offset.x;
        c_rect.y += offset.y;

        self::point_within(point, &c_rect)
    }

    pub fn draw(&self, ctx: &mut Context, offset: &Point) {
        let t_dest = Point {
            x: self.rect.x + offset.x,
            y: self.rect.y + offset.y,
        };
        let r = offset_rect(&self.rect, offset);
        graphics::rectangle(ctx, DrawMode::Line, r).unwrap();
        graphics::draw(ctx, &self.text, t_dest, 0.0).unwrap();
    }
}

pub trait UiState {
    fn draw(&self, ctx: &mut Context);
    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()>;
    fn hover(&mut self, point: &Point) -> Option<Rect>;
    fn return_state(&self) -> Option<SpriteType>;
}

pub struct AssetTypeUi {
    assets: Rc<Assets>,
    offset: Point,
    sub_ui: Box<UiState>,
    state: Option<SpriteData>,
    selected: Option<Rect>,
    hovered: Option<Rect>,
    object: SimpleButton,
    platform: SimpleButton,
    ground: SimpleButton,
    save: SimpleButton,
}

impl AssetTypeUi {
    pub fn new(
        ctx: &mut Context,
        assets: Rc<Assets>,
        offset: Point,
        data: Option<&SpriteData>,
    ) -> GameResult<AssetTypeUi> {
        let ui =
            AssetTypeUi::build_sub_ui(data.map(|d| &d.markers), &offset, ctx, assets.borrow())?;

        let obj_t = Text::new(ctx, "Object", &assets.font)?;
        let plat_t = Text::new(ctx, "Platform", &assets.font)?;
        let ground_t = Text::new(ctx, "Ground", &assets.font)?;

        let left_r = Rect::new(-H_SPACE, -W_SPACE, W_SIZE, H_SIZE);
        let h_center_r = Rect::new(0.0, -W_SPACE, W_SIZE, H_SIZE);
        let right_r = Rect::new(H_SPACE, -W_SPACE, W_SIZE, H_SIZE);

        let object = SimpleButton::new(obj_t, left_r);
        let platform = SimpleButton::new(plat_t, h_center_r);
        let ground = SimpleButton::new(ground_t, right_r);

        let save_t = Text::new(ctx, "Save", &assets.font)?;
        let save_r = Rect::new(1600.0 - 130.0, 1000.0 - 130.0, W_SIZE, H_SIZE);
        let save = SimpleButton::new(save_t, save_r);

        let selected = match ui.return_state() {
            Some(SpriteType::Ground{..}) => Some(ground.rect.clone()),
            Some(SpriteType::Platform{..}) => Some(platform.rect.clone()),
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
        assets: &Assets,
    ) -> GameResult<Box<UiState>> {
        match data.map(|d| d.clone()) {
            Some(markers) => {
                let mut sub_ui_offset = offset.clone();
                sub_ui_offset.y += 300.0;

                let ui: Box<UiState> = match markers {
                    SpriteType::Ground {
                        vertical: vert,
                        horizontal: hor,
                    } => {
                        let ground = GroundUi::new(ctx, assets, sub_ui_offset, (vert, hor))?;
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
        self.object.draw(ctx, &self.offset);
        self.platform.draw(ctx, &self.offset);
        self.ground.draw(ctx, &self.offset);
        self.save.draw(ctx, &Point::zero());

        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h).unwrap();
        };

        if let Some(s) = self.selected {
            let s = offset_rect(&s, &self.offset);
            draw_rect_with_outline(ctx, Color::new(0.0, 0.8, 0.2, 1.0), &s).unwrap();
        }

        self.sub_ui.draw(ctx);
    }

    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()> {
        if self.save.point_within(point, &Point::zero()) {
            return Err(GameError::from(String::from("Save now")));
        } else if self.object.point_within(point, &self.offset) {
            let ui = AssetTypeUi::build_sub_ui(
                Some(&SpriteType::Object),
                &self.offset,
                ctx,
                self.assets.borrow(),
            )?;
            self.selected = Some(self.object.rect.clone());
            self.sub_ui = ui;
        } else if self.ground.point_within(point, &self.offset) {
            let ui = AssetTypeUi::build_sub_ui(
                Some(&SpriteType::empty_ground()),
                &self.offset,
                ctx,
                self.assets.borrow(),
            )?;
            self.selected = Some(self.ground.rect.clone());
            self.sub_ui = ui;
        } else if self.platform.point_within(point, &self.offset) {
            let ui = AssetTypeUi::build_sub_ui(
                Some(&SpriteType::empty_platform()),
                &self.offset,
                ctx,
                self.assets.borrow(),
            )?;
            self.selected = Some(self.platform.rect.clone());
            self.sub_ui = ui;
        } else { self.sub_ui.interact(ctx, point)?; };
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let rect = vec![&self.object, &self.platform, &self.ground]
            .iter()
            .find(|b| b.point_within(point, &self.offset))
            .map(|b| b.rect.clone()).or_else(|| {
                if self.save.point_within(point, &Point::zero()) {
                    Some(self.save.rect.clone())
                } else { None }
            });
        let offseted = match rect {
            Some(mut r) => {
                r.x += self.offset.x;
                r.y += self.offset.y;
                Some(r)
            }
            None => None,
        };
        self.hovered = offseted;
        if let None = offseted {
            self.sub_ui.hover(point);
        };
        offseted
    }

    fn return_state(&self) -> Option<SpriteType> {
        self.sub_ui.return_state()
    }
}

pub struct GroundUi {
    offset: Point,
    hovered: Option<Rect>,
    state: (Vec<Vertical>, Vec<Horizontal>),
    left: SimpleButton,
    h_center: SimpleButton,
    right: SimpleButton,
    top: SimpleButton,
    v_center: SimpleButton,
    bottom: SimpleButton,
}

impl GroundUi {
    pub fn new(
        ctx: &mut Context,
        assets: &Assets,
        offset: Point,
        state: (Vec<Vertical>, Vec<Horizontal>),
    ) -> GameResult<GroundUi> {
        let left_t = Text::new(ctx, "Left", &assets.font)?;
        let right_t = Text::new(ctx, "Right", &assets.font)?;

        let middle_t = Text::new(ctx, "Middle", &assets.font)?;

        let top_t = Text::new(ctx, "Top", &assets.font)?;
        let bottom_t = Text::new(ctx, "Bottom", &assets.font)?;

        let left_r = Rect::new(-H_SPACE, -W_SPACE, W_SIZE, H_SIZE);
        let h_center_r = Rect::new(0.0, -W_SPACE, W_SIZE, H_SIZE);
        let right_r = Rect::new(H_SPACE, -W_SPACE, W_SIZE, H_SIZE);

        let top_r = Rect::new(0.0, 1.5 * W_SPACE, W_SIZE, H_SIZE);
        let v_center_r = Rect::new(0.0, 2.5 * W_SPACE, W_SIZE, H_SIZE);
        let bottom_r = Rect::new(0.0, 3.5 * W_SPACE, W_SIZE, H_SIZE);

        let left = SimpleButton::new(left_t, left_r);
        let h_center = SimpleButton::new(middle_t.clone(), h_center_r);
        let right = SimpleButton::new(right_t, right_r);

        let top = SimpleButton::new(top_t, top_r);
        let v_center = SimpleButton::new(middle_t, v_center_r);
        let bottom = SimpleButton::new(bottom_t, bottom_r);

        Ok(GroundUi {
            offset,
            left,
            h_center,
            right,
            top,
            v_center,
            bottom,
            hovered: None,
            state,
        })
    }

    pub fn to_vec(&self) -> Vec<&SimpleButton> {
        vec![
            &self.left,
            &self.right,
            &self.v_center,
            &self.h_center,
            &self.top,
            &self.bottom,
        ]
    }

    fn hor(&self, h: Horizontal) -> bool {
        self.state.1.contains(&h)
    }

    fn vert(&self, v: Vertical) -> bool {
        self.state.0.contains(&v)
    }

    fn draw_selected(&self, ctx: &mut Context) {
        let color = Color::new(0.0, 0.8, 0.2, 1.0);
        let offset = &self.offset;
        
        if self.hor(Horizontal::Left) {
            draw_rect_with_outline(ctx, color.clone(), &offset_rect(&self.left.rect, offset)).unwrap();
        };
        if self.hor(Horizontal::Center) {
            draw_rect_with_outline(ctx, color.clone(), &offset_rect(&self.h_center.rect, offset)).unwrap();
        };
        if self.hor(Horizontal::Right) {
            draw_rect_with_outline(ctx, color.clone(), &offset_rect(&self.right.rect, offset)).unwrap();
        };
        if self.vert(Vertical::Top) {
            draw_rect_with_outline(ctx, color.clone(), &offset_rect(&self.top.rect, offset)).unwrap();
        };
        if self.vert(Vertical::Center) {
            draw_rect_with_outline(ctx, color.clone(), &offset_rect(&self.v_center.rect, offset)).unwrap();
        };
        if self.vert(Vertical::Bottom) {
            draw_rect_with_outline(ctx, color.clone(), &offset_rect(&self.bottom.rect, offset)).unwrap();
        };
    }
}

impl UiState for GroundUi {
    fn draw(&self, ctx: &mut Context) {
        for b in self.to_vec().iter() {
            b.draw(ctx, &self.offset);
        }

        self.draw_selected(ctx);

        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h).unwrap();
        };
    }

    fn interact(&mut self, _: &mut Context, point: &Point) -> GameResult<()> {
        if self.left.point_within(point, &self.offset) {
            if !self.hor(Horizontal::Left) {
                distinct_vec_add(&mut self.state.1, Horizontal::Left);
            } else {
                self.state.1.retain(|h| {*h != Horizontal::Left});
            };
        } else if self.h_center.point_within(point, &self.offset) {
            if !self.hor(Horizontal::Center) {
                distinct_vec_add(&mut self.state.1, Horizontal::Center);
            } else {
                self.state.1.retain(|h| {*h != Horizontal::Center});
            };
        } else if self.right.point_within(point, &self.offset) {
            if !self.hor(Horizontal::Right) {
                distinct_vec_add(&mut self.state.1, Horizontal::Right);
            } else {
                self.state.1.retain(|h| {*h != Horizontal::Right});
            };
        } else if self.top.point_within(point, &self.offset) {
            if !self.vert(Vertical::Top) {
                distinct_vec_add(&mut self.state.0, Vertical::Top);
            } else {
                self.state.0.retain(|h| {*h != Vertical::Top});
            };
        } else if self.v_center.point_within(point, &self.offset) {
            if !self.vert(Vertical::Center) {
                distinct_vec_add(&mut self.state.0, Vertical::Center);
            } else {
                self.state.0.retain(|h| {*h != Vertical::Center});
            };
        } else if self.bottom.point_within(point, &self.offset) {
            if !self.vert(Vertical::Bottom) {
                distinct_vec_add(&mut self.state.0, Vertical::Bottom);
            } else {
                self.state.0.retain(|h| {*h != Vertical::Bottom});
            };
        }
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let rect = self.to_vec()
            .iter()
            .find(|b| b.point_within(point, &self.offset))
            .map(|b| b.rect.clone());
        let offseted = rect.map(|r| offset_rect(&r, &self.offset));
        self.hovered = offseted;
        offseted
    }

    fn return_state(&self) -> Option<SpriteType> {
        Some(SpriteType::Ground {
            horizontal: self.state.1.clone(),
            vertical: self.state.0.clone(),
        })
    }
}

pub struct PlatformUi {
    offset: Point,
    hovered: Option<Rect>,
    state: Vec<Horizontal>,
    left: SimpleButton,
    center: SimpleButton,
    right: SimpleButton,
}

impl PlatformUi {
    pub fn new(
        ctx: &mut Context,
        assets: &Assets,
        offset: Point,
        state: Vec<Horizontal>,
    ) -> GameResult<PlatformUi> {
        let left_t = Text::new(ctx, "Left", &assets.font)?;
        let right_t = Text::new(ctx, "Right", &assets.font)?;
        let middle_t = Text::new(ctx, "Middle", &assets.font)?;

        let left_r = Rect::new(-H_SPACE, -W_SPACE, W_SIZE, H_SIZE);
        let h_center_r = Rect::new(0.0, -W_SPACE, W_SIZE, H_SIZE);
        let right_r = Rect::new(H_SPACE, -W_SPACE, W_SIZE, H_SIZE);

        let left = SimpleButton::new(left_t, left_r);
        let center = SimpleButton::new(middle_t.clone(), h_center_r);
        let right = SimpleButton::new(right_t, right_r);

        Ok(PlatformUi {
            offset,
            hovered: None,
            state,
            left,
            center,
            right,
        })
    }

    pub fn to_vec(&self) -> Vec<&SimpleButton> {
        vec![&self.left, &self.center, &self.right]
    }

    fn hor(&self, h: Horizontal) -> bool {
        self.state.contains(&h)
    }

    fn draw_selected(&self, ctx: &mut Context) {
        let color = Color::new(0.0, 0.8, 0.2, 1.0);
        let offset = &self.offset;
        
        if self.hor(Horizontal::Left) {
            draw_rect_with_outline(ctx, color.clone(), &offset_rect(&self.left.rect, offset)).unwrap();
        };
        if self.hor(Horizontal::Center) {
            draw_rect_with_outline(ctx, color.clone(), &offset_rect(&self.center.rect, offset)).unwrap();
        };
        if self.hor(Horizontal::Right) {
            draw_rect_with_outline(ctx, color.clone(), &offset_rect(&self.right.rect, offset)).unwrap();
        };
    }
}

impl UiState for PlatformUi {
    fn draw(&self, ctx: &mut Context) {
        for b in self.to_vec().iter() {
            b.draw(ctx, &self.offset);
        }

        self.draw_selected(ctx);

        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h).unwrap();
        };
    }

    fn interact(&mut self, _: &mut Context, point: &Point) -> GameResult<()> {
        if self.left.point_within(point, &self.offset) {
            if !self.hor(Horizontal::Left) {
                distinct_vec_add(&mut self.state, Horizontal::Left);
            } else {
                self.state.retain(|h| {*h != Horizontal::Left});
            };
        } else if self.center.point_within(point, &self.offset) {
            if !self.hor(Horizontal::Center) {
                distinct_vec_add(&mut self.state, Horizontal::Center);
            } else {
                self.state.retain(|h| {*h != Horizontal::Center});
            };
        } else if self.right.point_within(point, &self.offset) {
            if !self.hor(Horizontal::Right) {
                distinct_vec_add(&mut self.state, Horizontal::Right);
            } else {
                self.state.retain(|h| {*h != Horizontal::Right});
            };
        }
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let rect = self.to_vec()
            .iter()
            .find(|b| b.point_within(point, &self.offset))
            .map(|b| b.rect.clone());
        let offseted = match rect {
            Some(mut r) => {
                r.x += self.offset.x;
                r.y += self.offset.y;
                Some(r)
            }
            None => None,
        };
        self.hovered = offseted;
        offseted
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

pub fn offset_rect(rect: &Rect, point: &Point) -> Rect {
    Rect {
        x: rect.x + point.x,
        y: rect.y + point.y,
        w: rect.w,
        h: rect.h,
    }
}
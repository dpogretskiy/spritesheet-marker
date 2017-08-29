
use ggez::graphics::*;
use ggez::graphics;
use ggez::{Context, GameResult};
use super::Assets;
use super::sprite::geom::Size;
use super::marker::*;
use std;
use std::collections::HashSet;
use std::cmp::PartialEq;
use std::hash::Hash;

const H_SPACE: f32 = 100.0;
const W_SPACE: f32 = 46.0;

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
        let Rect { x, y, w, h } = self.rect;
        let r = Rect {
            x: x + offset.x,
            y: y + offset.y,
            w,
            h,
        };
        graphics::rectangle(ctx, DrawMode::Line, r).unwrap();
        graphics::draw(ctx, &self.text, t_dest, 0.0).unwrap();
    }
}

pub trait UiState {
    fn draw(&self, ctx: &mut Context);
    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()>;
    fn hover(&mut self, point: &Point) -> Option<Rect>;
}

pub struct AssetTypeUi {
    offset: Point,
    sub_ui: Box<UiState>,
    state: Option<SpriteData>,
    hovered: Option<Rect>,
    ix: Option<usize>,
    object: SimpleButton,
    platform: SimpleButton,
    ground: SimpleButton,
}

impl AssetTypeUi {
    pub fn new(
        ctx: &mut Context,
        assets: &Assets,
        offset: Point,
        data: Option<&SpriteData>,
    ) -> GameResult<AssetTypeUi> {
        let ui = AssetTypeUi::build_sub_ui(data, &offset, ctx, assets)?;

        let obj_t = Text::new(ctx, "Object", &assets.font)?;
        let plat_t = Text::new(ctx, "Platform", &assets.font)?;
        let ground_t = Text::new(ctx, "Ground", &assets.font)?;

        let size = Size {
            w: (plat_t.width() + 15) as f32,
            h: (plat_t.height() + 15) as f32,
        };

        let left_r = Rect::new(-H_SPACE, -W_SPACE, size.w, size.h);
        let h_center_r = Rect::new(0.0, -W_SPACE, size.w, size.h);
        let right_r = Rect::new(H_SPACE, -W_SPACE, size.w, size.h);

        let object = SimpleButton::new(obj_t, left_r);
        let platform = SimpleButton::new(plat_t, h_center_r);
        let ground = SimpleButton::new(ground_t, right_r);

        Ok(AssetTypeUi {
            offset,
            sub_ui: ui,
            state: data.map(|d| d.clone()),
            hovered: None,
            ix: data.map(|d| d.index),
            object,
            platform,
            ground,
        })
    }

    fn build_sub_ui(data: Option<&SpriteData>, offset: &Point, ctx: &mut Context, assets: &Assets) -> GameResult<Box<UiState>> {
        match data {
            Some(data) => {
        let markers = data.markers.clone();

        let mut sub_ui_offset = offset.clone();
        sub_ui_offset.y += 300.0;

        let ui: Box<UiState> = match markers {
            SpriteType::Ground {
                vertical: vert,
                horizontal: hor,
            } => {
                let ground = GroundUi::new(ctx, assets, sub_ui_offset, (vert, hor))?;
                Box::new(ground)
            },
            SpriteType::Platform { horizontal: hor } => {
                let platform = PlatformUi::new(ctx, assets, sub_ui_offset, hor)?;
                Box::new(platform)
            },
            SpriteType::Object => Box::new(NoSubUi),
        };
        Ok(ui)
            },
            None => Ok(Box::new(NoSubUi)),
        }
    }
}

impl UiState for AssetTypeUi {
    fn draw(&self, ctx: &mut Context) {
        self.object.draw(ctx, &self.offset);
        self.platform.draw(ctx, &self.offset);
        self.ground.draw(ctx, &self.offset);

        self.sub_ui.draw(ctx);
    }

    fn interact(
        &mut self,
        ctx: &mut Context,
        point: &Point,
    ) -> GameResult<()> {
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let rect = vec![&self.object, &self.platform, &self.ground]
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

        let size = Size {
            w: (std::cmp::max(middle_t.width(), bottom_t.width()) + 15) as f32,
            h: (left_t.height() + 15) as f32,
        };

        let left_r = Rect::new(-H_SPACE, -W_SPACE, size.w, size.h);
        let h_center_r = Rect::new(0.0, -W_SPACE, size.w, size.h);
        let right_r = Rect::new(H_SPACE, -W_SPACE, size.w, size.h);

        let top_r = Rect::new(0.0, 3.0 * W_SPACE, size.w, size.h);
        let v_center_r = Rect::new(0.0, 4.0 * W_SPACE, size.w, size.h);
        let bottom_r = Rect::new(0.0, 5.0 * W_SPACE, size.w, size.h);

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
}

impl UiState for GroundUi {
    fn draw(&self, ctx: &mut Context) {
        for b in self.to_vec().iter() {
            b.draw(ctx, &self.offset);
        }

        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h);
        };
    }

    fn interact(&mut self, _: &mut Context, point: &Point) -> GameResult<()> {
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

        let size = Size {
            w: (middle_t.width() + 15) as f32,
            h: (left_t.height() + 15) as f32,
        };

        let left_r = Rect::new(-H_SPACE, -W_SPACE, size.w, size.h);
        let h_center_r = Rect::new(0.0, -W_SPACE, size.w, size.h);
        let right_r = Rect::new(H_SPACE, -W_SPACE, size.w, size.h);

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
}

impl UiState for PlatformUi {
    fn draw(&self, ctx: &mut Context) {
        for b in self.to_vec().iter() {
            b.draw(ctx, &self.offset);
        }
        if let Some(h) = self.hovered {
            draw_rect_with_outline(ctx, Color::new(0.8, 0.0, 0.0, 1.0), &h).unwrap();
        };
    }

    fn interact(&mut self, _: &mut Context, point: &Point) -> GameResult<()> {
        if self.left.point_within(point, &self.offset) {
            distinct_vec_add(&mut self.state, Horizontal::Left);
        } else if self.center.point_within(point, &self.offset) {
            distinct_vec_add(&mut self.state, Horizontal::Center);
        } else if self.right.point_within(point, &self.offset) {
            distinct_vec_add(&mut self.state, Horizontal::Right)
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
}

pub struct NoSubUi;
impl UiState for NoSubUi {
    fn draw(&self, ctx: &mut Context) {}

    fn interact(&mut self, ctx: &mut Context, point: &Point) -> GameResult<()> {
        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        None
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


use ggez::graphics::*;
use ggez::graphics;
use ggez::{Context, GameResult};
use super::Assets;
use super::sprite::geom::Size;
use std;

const H_SPACE: f32 = 100.0;
const W_SPACE: f32 = 46.0;

pub struct SimpleButton {
    pub rect: Rect,
    pub text: Text,
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
    fn interact(
        &mut self,
        ctx: &mut Context,
        state: &mut super::Game,
        point: &Point,
    ) -> GameResult<()>;
    fn hover(&mut self, point: &Point) -> Option<Rect>;
}

pub struct AssetTypeUi<T: UiState> {
    state: Box<T>,

    animation: SimpleButton,
    object: SimpleButton,
    platform: SimpleButton,
    ground: SimpleButton,
}

pub struct GroundUi {
    offset: Point,
    hovered: Option<Rect>,
    left: SimpleButton,
    h_center: SimpleButton,
    right: SimpleButton,
    top: SimpleButton,
    v_center: SimpleButton,
    bottom: SimpleButton,
}

impl GroundUi {
    pub fn new(ctx: &mut Context, assets: &Assets, offset: Point) -> GameResult<GroundUi> {
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

    fn interact(
        &mut self,
        ctx: &mut Context,
        state: &mut super::Game,
        point: &Point,
    ) -> GameResult<()> {
        let buttons = self.to_vec();

        let pressed = buttons.iter().enumerate()
            .find(|&(ix, button)| button.point_within(point, &self.offset));

        Ok(())
    }

    fn hover(&mut self, point: &Point) -> Option<Rect> {
        let rect = self.to_vec().iter().find(|b| b.point_within(point, &self.offset)).map(|b| b.rect.clone());
        let offseted = match rect {
            Some(mut r) => {
                r.x += self.offset.x;
                r.y += self.offset.y;
                Some(r)
            },
            None => None,
        };
        self.hovered = offseted;
        offseted 
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

use ggez::graphics::*;
use ggez::graphics;
use ggez::{Context, GameError, GameResult};
use super::Assets;

pub struct SimpleButton {
    pub rect: Rect,
    pub text: Text,
}

impl SimpleButton {
    pub fn new(text: Text, rect: Rect) -> SimpleButton {
        SimpleButton { rect, text }
    }

    pub fn point_within(&self, point: &Point) -> bool {
        self::point_within(point, &self.rect)
    }

    pub fn draw(&self, ctx: &mut Context, offset: &Point) {
        let t_dest = Point {
            x : self.rect.x + offset.x,
            y: self.rect.y + offset.y,
        };
        let Rect {x, y, w, h} = self.rect;
        let r = Rect {
            x: x + offset.x,
            y: y + offset.y,
            w, h
        };
        graphics::rectangle(ctx, DrawMode::Line, r);
        graphics::draw(ctx, &self.text, t_dest, 0.0);
    }
}

pub struct GroundUi {
    left: SimpleButton,
    h_center: SimpleButton,
    right: SimpleButton,
    top: SimpleButton,
    v_center: SimpleButton,
    bottom: SimpleButton,
}

const SPACE: f32 = 100.0;

impl GroundUi {
    pub fn new(ctx: &mut Context, assets: &Assets) -> GameResult<GroundUi> {
        let left_t = Text::new(ctx, "Left", &assets.font)?;
        let left_r = Rect::new(
            -SPACE,
            -SPACE,
            (left_t.width() + 15) as f32,
            (left_t.height() + 10) as f32,
        );

        let middle_t = Text::new(ctx, "Middle", &assets.font)?;
        let h_center_r = Rect::new(
            0.0,
            -SPACE,
            (middle_t.width() + 15) as f32,
            (middle_t.height() + 10) as f32,
        );
        let v_center_r = Rect::new(
            0.0,
            SPACE,
            (middle_t.width() + 15) as f32,
            (middle_t.height() + 10) as f32,
        );

        let right_t = Text::new(ctx, "Right", &assets.font)?;
        let right_r = Rect::new(
            SPACE,
            -SPACE,
            (right_t.width() + 15) as f32,
            (right_t.height() + 10) as f32,
        );

        let top_t = Text::new(ctx, "Top", &assets.font)?;
        let top_r = Rect::new(
            -SPACE,
            SPACE,
            (top_t.width() + 15) as f32,
            (top_t.height() + 10) as f32,
        );

        let bottom_t = Text::new(ctx, "Bottom", &assets.font)?;
        let bottom_r = Rect::new(
            SPACE,
            SPACE,
            (bottom_t.width() + 15) as f32,
            (bottom_t.height() + 10) as f32
        );


        let left = SimpleButton::new(left_t, left_r);
        let h_center = SimpleButton::new(middle_t.clone(), h_center_r);
        let right = SimpleButton::new(right_t, right_r);

        let top = SimpleButton::new(top_t, top_r);
        let v_center = SimpleButton::new(middle_t, v_center_r);
        let bottom = SimpleButton::new(bottom_t, bottom_r);

        Ok(GroundUi {
            left, h_center, right, top, v_center, bottom
        })
    }

    fn to_array(&self) -> [&SimpleButton; 6] {
        [&self.left, &self.right, &self.v_center, &self.h_center, &self.top, &self.bottom]
    }

    pub fn draw(&self, ctx: &mut Context, offset: &Point) {
        for b in self.to_array().iter() {
            b.draw(ctx, offset);
        }
    }
}


fn ui_stuff(assets: &super::Assets, ctx: &mut Context) {}

pub fn point_within(point: &Point, rect: &Rect) -> bool {
        let &Point { x, y } = point;
        rect.bottom() < y && rect.top() > y && rect.left() < x && rect.right() > x
}
mod color_picker;
mod fractal;
mod fractal_widget;
mod image_utils;
mod renderer;

use std::f64::consts::PI;
use druid::kurbo::{Circle, Line, Rect};
use druid::widget::{prelude::*, Label};
use druid::{AppLauncher, Color, LocalizedString, Point, Vec2, WindowDesc};
use color_picker::{color_picker, HSL};
use fractal_widget::{FractalData, FractalWidget};

struct AnimWidget {
    t: f64,
}

impl Widget<()> for AnimWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                self.t = 0.0;
                ctx.request_anim_frame();
            }
            Event::AnimFrame(interval) => {
                ctx.request_paint();
                self.t += f64::min(*interval as f64, 5e7) * 1e-9;
                if self.t < 1.0 {
                    ctx.request_anim_frame();
                } else {
                    self.t = 0.0;
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &(), _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &(), _data: &(), _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &(),
        _env: &Env,
    ) -> Size {
        bc.constrain((100.0, 100.0))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &(), _env: &Env) {
        let t = self.t;
        let center = Point::new(50.0, 50.0);
        ctx.paint_with_z_index(1, move |ctx| {
            let ambit = center + 45.0 * Vec2::from_angle((0.75 + t) * 2.0 * PI);
            ctx.stroke(Line::new(center, ambit), &Color::WHITE, 1.0);
        });

        ctx.fill(Circle::new(center, 50.0), &Color::BLACK);
    }
}

pub fn main() {
    let window = WindowDesc::new(FractalWidget::new())
        .title("Fractal renderer")
        .window_size((512., 512.));
    // let data = HSL::new();
    AppLauncher::with_window(window)
        .log_to_console()
        .launch(FractalData::new())
        .expect("launch failed");
}

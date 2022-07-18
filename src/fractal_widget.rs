use crate::fractal::*;
use crate::image_utils::{IPoint, RGB};
use crate::renderer::*;
use druid::kurbo::{Circle, Rect};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::platform_menus::mac::file::print;
use druid::widget::{prelude::*, Align, Flex};
use druid::widget::{Label, Slider};
use druid::{Color, Lens, Point, WidgetExt};

const MAX_RADIUS: f64 = 2.;

#[derive(Clone, Data, Lens)]
pub struct FractalData {
    focus: Circle,
    selection: Rect,
}

impl FractalData {
    pub fn new() -> Self {
        FractalData {
            focus: Circle {
                center: Point::new(-0.5, 0.),
                radius: MAX_RADIUS,
            },
            selection: Rect::ZERO,
        }
    }
}

pub struct FractalWidget {
    size: Size,
    renderer: Renderer,
    image: Vec<RGB>,
    image_data: Vec<u8>,
}

impl FractalWidget {
    pub fn new() -> Self {
        FractalWidget {
            size: Size::ZERO,
            renderer: Renderer::new(8),
            image: Vec::new(),
            image_data: Vec::new(),
        }
    }
}

impl Widget<FractalData> for FractalWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FractalData, _env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                // // TEMP
                // println!("Stopping...");
                // self.renderer.stop();
                // println!("Stopped!");
                // //
                ctx.set_active(true);
                data.selection.x0 = mouse.pos.x;
                data.selection.y0 = mouse.pos.y;
                data.selection.x1 = mouse.pos.x;
                data.selection.y1 = mouse.pos.y;
                ctx.request_paint();
            }
            Event::MouseMove(mouse) => {
                if ctx.is_active() {
                    data.selection.x1 = mouse.pos.x;
                    data.selection.y1 = mouse.pos.y;
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse) => {
                ctx.set_active(false);
                if data.selection.area() < 4. {
                    // Unzoom
                    let p0 = px_to_world(
                        &data.focus,
                        &self.size,
                        &IPoint {
                            x: data.selection.x0 as usize,
                            y: data.selection.y0 as usize,
                        },
                    );
                    data.focus.center = Point { x: p0.x, y: p0.y };
                    data.focus.radius = data.focus.radius * 2.;
                    if data.focus.radius > MAX_RADIUS {
                        data.focus.radius = MAX_RADIUS;
                        data.focus.center.x = -0.5;
                        data.focus.center.y = 0.;
                    }
                } else {
                    // Zoom
                    let p0 = px_to_world(
                        &data.focus,
                        &self.size,
                        &IPoint {
                            x: data.selection.x0 as usize,
                            y: data.selection.y0 as usize,
                        },
                    );
                    let p1 = px_to_world(
                        &data.focus,
                        &self.size,
                        &IPoint {
                            x: data.selection.x1 as usize,
                            y: data.selection.y1 as usize,
                        },
                    );
                    data.focus.center = Point {
                        x: (p0.x + p1.x) / 2.,
                        y: (p0.y + p1.y) / 2.,
                    };
                    data.focus.radius = (p0.x - p1.x).abs().min((p0.y - p1.y).abs()) / 2.;
                }
                // Clear image
                self.image.fill(RGB::TRANSPARENT);
                ctx.request_paint();
            }
            Event::AnimFrame(interval) => {
                // Anim frame requested
                ctx.request_anim_frame();
                ctx.request_paint();
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &FractalData,
        _env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.request_anim_frame();
            }
            LifeCycle::Size(_) => {
                self.image.fill(RGB::TRANSPARENT);
            }
            _ => {}
        }
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &FractalData,
        _data: &FractalData,
        _env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &FractalData,
        _env: &Env,
    ) -> Size {
        let default_size = Size::new(512., 512.);
        self.size = bc.constrain(default_size);
        self.size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FractalData, _env: &Env) {
        // Render fractal
        // self.size =
        // self.size.x = ctx.size().width as usize;
        // self.size.y = ctx.size().height as usize;
        // self.image_data = create_fractal(data, &self.size);
        // let (w, h) = (ctx.size().width as usize, ctx.size().height as usize);

        self.renderer.resize(ctx.size(), data.focus);
        let result = self.renderer.update(&mut self.image);
        println!("progress: {}", result.progress);

        if !self.image.is_empty() {
            RGB::create_image_data(&self.image, &mut self.image_data);
            let image = ctx
                .make_image(
                    result.image_size.x,
                    result.image_size.y,
                    &self.image_data,
                    ImageFormat::RgbaSeparate,
                )
                .unwrap();
            let size = ctx.size();
            ctx.draw_image(
                &image,
                Rect::from_origin_size(Point::ORIGIN, size),
                InterpolationMode::Bilinear,
            );
        }

        // Draw selection
        if ctx.is_active() {
            let stroke_color = Color::WHITE;
            let fill_color = Color::BLACK.with_alpha(0.2);
            ctx.fill(data.selection, &fill_color);
            ctx.stroke(data.selection, &stroke_color, 1.);
        }
    }
}

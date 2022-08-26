use crate::fractal::*;
use crate::image_utils::{IPoint, RGB};
use crate::renderer::Renderer;
use druid::kurbo::{Circle, Rect};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::platform_menus::mac::file::print;
use druid::widget::prelude::*;
use druid::{Code, Color, Key, Lens, MouseButton, Point};

const MAX_RADIUS: f64 = 2.;

#[derive(Clone, Data, Lens)]
pub struct FractalData {
    focus: Circle,
    selection: Rect,
    progress: f64,
}

impl FractalData {
    pub fn new() -> Self {
        let mut instance = FractalData {
            focus: Circle::new(Point::ZERO, 0.),
            selection: Rect::ZERO,
            progress: 0.,
        };
        instance.zoom_reset();
        instance
    }

    fn clip_zoom(&mut self) {
        if self.focus.radius > MAX_RADIUS {
            self.focus.radius = MAX_RADIUS;
            self.focus.center.x = -0.5;
            self.focus.center.y = 0.;
        }
    }

    pub fn zoom_reset(&mut self) {
        self.focus = Circle {
            center: Point::new(-0.5, 0.),
            radius: MAX_RADIUS,
        };
        self.selection = Rect::ZERO;
    }

    pub fn zoom_center(&mut self, factor: f64) {
        self.focus.radius = self.focus.radius / factor;
        self.clip_zoom();
    }

    pub fn zoom_point(&mut self, size: &Size, point: &IPoint, factor: f64) {
        // Unzoom
        let p0 = px_to_world(&self.focus, size, point);
        self.focus.center = Point { x: p0.x, y: p0.y };
        self.focus.radius = self.focus.radius / factor;
        self.clip_zoom();
    }

    pub fn zoom_rect(&mut self, size: &Size, selection: Rect) {
        // Zoom
        let p0 = px_to_world(
            &self.focus,
            size,
            &IPoint {
                x: selection.x0 as usize,
                y: selection.y0 as usize,
            },
        );
        let p1 = px_to_world(
            &self.focus,
            size,
            &IPoint {
                x: selection.x1 as usize,
                y: selection.y1 as usize,
            },
        );
        self.focus.center = Point {
            x: (p0.x + p1.x) / 2.,
            y: (p0.y + p1.y) / 2.,
        };
        self.focus.radius = (p0.x - p1.x).abs().min((p0.y - p1.y).abs()) / 2.;
    }

    pub fn zoom_factor_str(&self) -> String {
        format!("{}x", (MAX_RADIUS / self.focus.radius).round())
    }
}

pub struct FractalWidget {
    size: Size,
    renderer: Renderer,
    image: Vec<RGB>,
    image_data: Vec<u8>,
    progress: f64,
    drag_center: Option<Point>,
}

impl FractalWidget {
    pub fn new() -> Self {
        FractalWidget {
            size: Size::ZERO,
            renderer: Renderer::new(),
            image: Vec::new(),
            image_data: Vec::new(),
            progress: 0.,
            drag_center: None,
        }
    }
}

fn swap(a: &mut f64, b: &mut f64) {
    let _b = *b;
    *b = *a;
    *a = _b;
}

impl Widget<FractalData> for FractalWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FractalData, _env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                ctx.set_active(true);
                data.selection.x0 = mouse.pos.x;
                data.selection.y0 = mouse.pos.y;
                data.selection.x1 = mouse.pos.x;
                data.selection.y1 = mouse.pos.y;
                ctx.request_paint();
            }
            Event::KeyDown(key_event) => {
                if key_event.code == Code::ShiftLeft || key_event.code == Code::ShiftRight {
                    self.drag_center = Option::Some(data.focus.center);
                }
            }
            Event::MouseMove(mouse) => {
                if ctx.is_active() {
                    if let Some(center) = self.drag_center {
                        let p0 = IPoint {
                            x: data.selection.x0 as usize,
                            y: data.selection.y0 as usize,
                        };
                        let p1 = IPoint {
                            x: data.selection.x1 as usize,
                            y: data.selection.y1 as usize,
                        };
                        let w0 = px_to_world(&data.focus, &ctx.size(), &p0);
                        let w1 = px_to_world(&data.focus, &ctx.size(), &p1);

                        data.focus.center = Point {
                            x: center.x - w1.x + w0.x,
                            y: center.y - w1.y + w0.y,
                        };
                    }
                    data.selection.x1 = mouse.pos.x;
                    data.selection.y1 = mouse.pos.y;
                    ctx.request_paint();
                }
            }
            Event::MouseUp(_mouse) => {
                ctx.set_active(false);
                if self.drag_center.is_none() {
                    // Update selection
                    if data.selection.x1 < data.selection.x0 {
                        swap(&mut data.selection.x0, &mut data.selection.x1);
                    }
                    if data.selection.y1 < data.selection.y0 {
                        swap(&mut data.selection.y0, &mut data.selection.y1);
                    }
                    if data.selection.area() < 4. {
                        let point = IPoint {
                            x: data.selection.x0 as usize,
                            y: data.selection.y0 as usize,
                        };
                        data.zoom_point(&self.size, &point, 0.5);
                    } else {
                        data.zoom_rect(&self.size, data.selection);
                    }
                }
                // Cancel drag
                self.drag_center = None;
                // Clear image
                self.image.fill(RGB::TRANSPARENT);
                ctx.request_paint();
            }
            Event::AnimFrame(_interval) => {
                // Populate progress
                data.progress = self.progress;
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
        self.size = bc.max();
        self.size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FractalData, _env: &Env) {
        // Render fractal
        self.renderer.resize(ctx.size(), data.focus);
        let result = self.renderer.update(&mut self.image);
        self.progress = result.progress;

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
        if ctx.is_active() && self.drag_center.is_none() {
            let stroke_color = Color::WHITE;
            let fill_color = Color::BLACK.with_alpha(0.2);
            ctx.fill(data.selection, &fill_color);
            ctx.stroke(data.selection, &stroke_color, 1.);
        }
    }
}

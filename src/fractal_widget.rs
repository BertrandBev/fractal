use crate::fractal::*;
use crate::image_utils::RGB;
use crate::renderer::*;
use druid::kurbo::Rect;
use druid::piet::{ImageFormat, InterpolationMode};
use druid::platform_menus::mac::file::print;
use druid::widget::{prelude::*, Align, Flex};
use druid::widget::{Label, Slider};
use druid::{Color, Lens, Point, WidgetExt};

#[derive(Clone, Data, Lens)]
pub struct FractalData {
    pub center: Point,
    pub radius: f64,
    // Selection
    selection: Rect,
}

impl FractalData {
    pub fn new() -> Self {
        FractalData {
            center: Point::new(-0.5, 0.),
            radius: 2.,
            selection: Rect::ZERO,
        }
    }
}

fn px_to_world(data: &FractalData, size: &IPoint, point: &IPoint) -> Point {
    //
    let mut xr = point.x as f64 / size.x as f64;
    let mut yr = point.y as f64 / size.y as f64;
    // Now transform back to window
    xr = (xr * 2. - 1.) * data.radius;
    yr = (yr * 2. - 1.) * data.radius;
    if size.x > size.y {
        xr *= size.x as f64 / size.y as f64;
    } else {
        yr *= size.y as f64 / size.x as f64;
    }
    Point {
        x: xr + data.center.x,
        y: yr + data.center.y,
    }
}

fn create_fractal(data: &FractalData, size: &IPoint) -> Vec<u8> {
    let mut image_data = vec![0; size.x * size.y * 4];
    let mut idx = 0;

    let f = (0.001 + 2.0 * data.radius).sqrt();
    let max_iter = (223.0 / f).floor() as usize;
    println!("max_iter: {}", max_iter);

    for y in 0..size.y {
        for x in 0..size.x {
            // Iteration counter
            let world = px_to_world(data, size, &IPoint { x: x, y: y });

            let res = mandelbrot(
                Complex {
                    r: world.x,
                    i: world.y,
                },
                100.,
                max_iter,
            );
            let rgb = color_scheme(&res);
            image_data[idx + 0] = rgb.r;
            image_data[idx + 1] = rgb.g;
            image_data[idx + 2] = rgb.b;
            image_data[idx + 3] = 255;
            idx += 4;
        }
    }
    image_data
}

pub struct FractalWidget {
    size: IPoint,
    renderer: Renderer,
    image: Vec<RGB>,
    image_data: Vec<u8>,
}

impl FractalWidget {
    pub fn new() -> Self {
        FractalWidget {
            size: IPoint::default(),
            renderer: Renderer::new(4),
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
                    return;
                }

                let p0 = px_to_world(
                    data,
                    &self.size,
                    &IPoint {
                        x: data.selection.x0 as usize,
                        y: data.selection.y0 as usize,
                    },
                );
                let p1 = px_to_world(
                    data,
                    &self.size,
                    &IPoint {
                        x: data.selection.x1 as usize,
                        y: data.selection.y1 as usize,
                    },
                );
                data.center = Point {
                    x: (p0.x + p1.x) / 2.,
                    y: (p0.y + p1.y) / 2.,
                };
                data.radius = (p0.x - p1.x).abs().min((p0.y - p1.y).abs());

                // data.center = world;
                // data.radius = data.radius / 2.;
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
        println!("update!");
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &FractalData,
        _env: &Env,
    ) -> Size {
        let default_size = Size::new(512., 512.);
        let size = bc.constrain(default_size);
        self.size.x = size.width as usize;
        self.size.y = size.height as usize;
        // dbg!(size);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FractalData, _env: &Env) {
        // Render fractal
        // self.size =
        // self.size.x = ctx.size().width as usize;
        // self.size.y = ctx.size().height as usize;
        // self.image_data = create_fractal(data, &self.size);
        // let (w, h) = (ctx.size().width as usize, ctx.size().height as usize);

        // Get image
        let region = Rect::ZERO;
        self.renderer.resize(ctx.size(), region);
        let (w, h) = self.renderer.get_image(&mut self.image);

        if !self.image.is_empty() {
            RGB::create_image_data(&self.image, &mut self.image_data);
            let image = ctx
                .make_image(w, h, &self.image_data, ImageFormat::RgbaSeparate)
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

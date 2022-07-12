use druid::kurbo::Rect;
use druid::piet::{ImageFormat, InterpolationMode};
use druid::widget::{prelude::*, Align, Flex};
use druid::widget::{Label, Slider};
use druid::{Color, Lens, Point, WidgetExt};

fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
    let mut t = t;
    if t < 0. {
        t += 1.
    }
    if t > 1. {
        t -= 1.
    };
    if t < 1. / 6. {
        return p + (q - p) * 6. * t;
    }
    if t < 1. / 2. {
        return q;
    }
    if t < 2. / 3. {
        return p + (q - p) * (2. / 3. - t) * 6.;
    }
    return p;
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let r;
    let g;
    let b;

    if s == 0.0 {
        r = l;
        g = l;
        b = l; // achromatic
    } else {
        let q = if l < 0.5 { l * (1. + s) } else { l + s - l * s };

        let p = 2. * l - q;
        r = hue_to_rgb(p, q, h + 1. / 3.);
        g = hue_to_rgb(p, q, h);
        b = hue_to_rgb(p, q, h - 1. / 3.);
    }

    return (
        (r * 255.).round() as u8,
        (g * 255.).round() as u8,
        (b * 255.).round() as u8,
    );
}

fn make_sl_image(width: usize, height: usize, hue: f64) -> Vec<u8> {
    let mut image_data = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let ix = (y * width + x) * 4;
            let x_ratio = x as f64 / width as f64;
            let y_ratio = y as f64 / width as f64;

            // Where the magic happens
            let color = hsl_to_rgb(hue, x_ratio, y_ratio);

            image_data[ix + 0] = color.0;
            image_data[ix + 1] = color.1;
            image_data[ix + 2] = color.2;
            image_data[ix + 3] = 255
        }
    }

    image_data
}

#[derive(Clone, Data, Lens)]
pub struct HSL {
    hue: f64,
    saturation: f64,
    lightness: f64,
}

impl HSL {
    pub fn new() -> Self {
        HSL {
            hue: 0.4,
            saturation: 0.4,
            lightness: 0.4,
        }
    }
}

struct ColorPicker {
    size: Size,
}

impl ColorPicker {
    fn new() -> Self {
        ColorPicker {
            size: Size::default(),
        }
    }
}

impl Widget<HSL> for ColorPicker {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut HSL, _env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                ctx.set_active(true);
                data.saturation = (mouse.pos.x / self.size.width).max(0.0).min(1.0);
                data.lightness = (mouse.pos.y / self.size.height).max(0.0).min(1.0);
                ctx.request_paint();
            }
            Event::MouseMove(mouse) => {
                if ctx.is_active() {
                    data.saturation = (mouse.pos.x / self.size.width).max(0.0).min(1.0);
                    data.lightness = (mouse.pos.y / self.size.height).max(0.0).min(1.0);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(_mouse) => {
                ctx.set_active(false);
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &HSL, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &HSL, _data: &HSL, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &HSL,
        _env: &Env,
    ) -> Size {
        let default_size = Size::new(256., 256.);
        self.size = bc.constrain(default_size);
        self.size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &HSL, _env: &Env) {
        // We're generating a 256 by 256 pixels image, with a constant hue of 0.5
        let image_data = make_sl_image(256, 256, data.hue);

        let image = ctx
            .make_image(256, 256, &image_data, ImageFormat::RgbaSeparate)
            .unwrap();

        // When piet draws our image it will stretch it automatically.
        // We'll fix this later by giving our widget a fixed size.
        let size = ctx.size();
        ctx.draw_image(
            &image,
            Rect::from_origin_size(Point::ORIGIN, size),
            InterpolationMode::Bilinear,
        );

        //
        // Create a UnitPoint from our cursor floats
        let cursor_point = druid::piet::UnitPoint::new(data.saturation, data.lightness);
        // Create a rect that's the size of our whole widget
        let resolve_rect = Rect::from_origin_size(Point::ORIGIN, ctx.size());
        // Calling resolve on the UnitPoint returns a Point relative to the rectangle it's passed
        let resolved_point = cursor_point.resolve(resolve_rect);
        let cursor_rect = Rect::from_origin_size(resolved_point, (10., 10.));
        ctx.stroke(cursor_rect, &Color::BLACK, 1.0);

        let inset_point = resolved_point + druid::kurbo::Vec2::new(1., 1.);
        let white_cursor_rect = Rect::from_origin_size(inset_point, (8., 8.));
        ctx.stroke(white_cursor_rect, &Color::rgba8(255, 255, 255, 128), 1.0);
    }

    // fn paint(&mut self, ctx: &mut PaintCtx, _data: &(), _env: &Env) {
    //     let rgb = hsl_to_rgb(0.5, 0.5, 0.5);
    //     let rect = Rect::from_origin_size(Point::ORIGIN, base_state.size());
    //     ctx.fill(rect, &Color::rgb8(rgb.0, rgb.1, rgb.2));
    // }
}

pub fn color_picker() -> impl Widget<HSL> {
    let mut col = Flex::column();
    let slider = Slider::new().lens(HSL::hue);
    let hue_label = Label::new(|data: &HSL, _env: &_| format!("Hue: {0:.2}", data.hue));
    let sat_label = Label::new(|data: &HSL, _env: &_| format!("Sat: {0:.2}", data.saturation));
    let light_label = Label::new(|data: &HSL, _env: &_| format!("Light: {0:.2}", data.lightness));
    col.add_child(slider);
    col.add_child(hue_label);
    col.add_child(sat_label);
    col.add_child(light_label);
    let picker = ColorPicker::new();
    col.add_child(Align::centered(picker));
    col
}

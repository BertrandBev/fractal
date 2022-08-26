mod fractal;
mod fractal_widget;
mod image_utils;
mod progress_bar;
mod renderer;
mod stack_widget;
mod time;
use progress_bar::ProgressBar;
use wasm_bindgen::prelude::*;

use druid::{
    widget::{Button, Flex, Label, SizedBox},
    AppLauncher, Color, FontDescriptor, FontFamily, FontStyle, UnitPoint, Widget, WidgetExt,
    WindowDesc,
};
use fractal_widget::{FractalData, FractalWidget};
use stack_widget::{StackAlign, StackWidget};

pub fn build_gui() -> impl Widget<FractalData> {
    let fractal_widget = FractalWidget::new();

    let zoom_in = Button::<FractalData>::new("+")
        .padding(5.0)
        .on_click(|_ctx, data, _env| {
            data.zoom_center(2.0);
        });
    let zoom_out = Button::<FractalData>::new("-")
        .padding(5.0)
        .on_click(|_ctx, data, _env| {
            data.zoom_center(0.5);
        });
    let reset = Button::<FractalData>::new("reset")
        .padding(5.0)
        .on_click(|_ctx, data, _env| {
            data.zoom_reset();
        });
    let label = Label::new(|data: &FractalData, _: &_| data.zoom_factor_str());

    let font = FontDescriptor::new(FontFamily::SANS_SERIF).with_style(FontStyle::Italic);
    let credits = Label::new("fractal.rs by bbev")
        .with_text_size(10.)
        .with_text_color(Color::WHITE.with_alpha(0.5))
        .with_font(font);

    let progress_bar = Flex::row()
        .with_flex_child(ProgressBar::new().lens(FractalData::progress).expand(), 1.0)
        .background(Color::RED)
        .fix_height(4.);
    let button_bar = Flex::row()
        .with_child(zoom_in)
        .with_child(zoom_out)
        .with_child(reset)
        .with_child(label)
        .with_flex_spacer(1.)
        .with_child(credits)
        .padding(10.);

    let toolbar = Flex::column()
        .with_child(progress_bar)
        .with_child(button_bar.expand_width())
        .background(Color::rgba8(0, 0, 0, 128));

    // widget
    Flex::column()
        .with_flex_child(fractal_widget.expand(), 1.0)
        .with_child(toolbar)
    // .with_child(button)
}

// Wasm wrapper
#[wasm_bindgen]
pub fn wasm_main() {
    // This hook is necessary to get panic messages in the console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    main()
}

pub fn main() {
    // RGB::resize_image(&[], IPoint::default(), &mut Vec::new(), IPoint::new(3, 2));
    let window = WindowDesc::new(build_gui())
        .title("Fractal renderer")
        // .window_size((512., 256.));
        .window_size((512., 512.));
    // let data = HSL::new();
    AppLauncher::with_window(window)
        .log_to_console()
        .launch(FractalData::new())
        .expect("launch failed");
}

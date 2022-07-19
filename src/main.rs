mod color_picker;
mod fractal;
mod fractal_widget;
mod image_utils;
mod renderer;
mod stack_widget;
use wasm_bindgen::prelude::*;

use druid::{
    widget::{Button, SizedBox},
    AppLauncher, Widget, WidgetExt, WindowDesc,
};
use fractal_widget::{FractalData, FractalWidget};
use stack_widget::{StackAlign, StackWidget};

pub fn build_gui() -> impl Widget<FractalData> {
    let fractal_widget = FractalWidget::new();

    let button = Button::<FractalData>::new("increment")
        .padding(5.0)
        .align_right();
    // let sized = SizedBox::new(button).width(128.).height(64.);

    let widget = StackWidget::new()
        .with_child(fractal_widget, StackAlign::TopLeft)
        .with_child(button, StackAlign::BottomCenter);
    widget
}

// This wrapper function is the primary modification we're making to the vanilla
// hello.rs example.
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

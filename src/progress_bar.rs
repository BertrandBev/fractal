use druid::widget::prelude::*;
use druid::{Color, Event, EventCtx};

#[derive(Debug, Clone, Default)]
pub struct ProgressBar;

impl ProgressBar {
    /// Return a new `ProgressBar`.
    pub fn new() -> ProgressBar {
        Self::default()
    }
}

impl Widget<f64> for ProgressBar {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut f64, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &f64, _env: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &f64, _data: &f64, _env: &Env) {
        ctx.request_paint();
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &f64,
        env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, env: &Env) {
        let clamped = data.max(0.0).min(1.0);
        let size = ctx.size();
        let rect = Size::new(size.width, size.height).to_rect();

        // Background
        ctx.fill(rect, &Color::GRAY);

        // Bar
        let calculated_bar_width = clamped * rect.width();
        let rect = Size::new(calculated_bar_width, size.height).to_rect();
        ctx.fill(rect, &Color::BLUE.with_alpha(0.5));
    }
}

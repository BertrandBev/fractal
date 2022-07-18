use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Size, UpdateCtx, Widget, WidgetPod,
};

pub enum StackAlign {
    LeftCenter,
    TopCenter,
    RightCenter,
    BottomCenter,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct StackWidget<T> {
    widgets: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
    aligns: Vec<StackAlign>,
}

impl<T> StackWidget<T> {
    pub fn new() -> Self {
        StackWidget {
            widgets: vec![],
            aligns: vec![],
        }
    }

    pub fn with_child(mut self, widget: impl Widget<T> + 'static, align: StackAlign) -> Self {
        self.widgets.push(WidgetPod::new(Box::new(widget)));
        self.aligns.push(align);
        self
    }
}

impl<T> Default for StackWidget<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Data> Widget<T> for StackWidget<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for widget in self.widgets.iter_mut() {
            widget.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for widget in self.widgets.iter_mut() {
            widget.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        for widget in self.widgets.iter_mut() {
            widget.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let mut sizes: Vec<Size> = vec![];
        let mut max_size: Size = Size::ZERO;
        for widget in self.widgets.iter_mut() {
            //  &BoxConstraints::new(self.cell_size, self.cell_size)
            let size = widget.layout(ctx, bc, data, env);
            sizes.push(size);
            max_size.width = max_size.width.max(size.width);
            max_size.height = max_size.height.max(size.height);
        }
        for (idx, widget) in self.widgets.iter_mut().enumerate() {
            let align = &self.aligns[idx];
            let size = sizes[idx];
            let (x, y) = match align {
                StackAlign::LeftCenter => (0., (max_size.height - size.height) / 2.),
                StackAlign::TopCenter => ((max_size.width - size.width) / 2., 0.),
                StackAlign::RightCenter => (
                    max_size.width - size.width,
                    (max_size.height - size.height) / 2.,
                ),
                StackAlign::BottomCenter => (
                    (max_size.width - size.width) / 2.,
                    max_size.height - size.height,
                ),
                StackAlign::TopLeft => (0., 0.),
                StackAlign::TopRight => (max_size.width - size.width, 0.),
                StackAlign::BottomLeft => (0., max_size.height - size.height),
                StackAlign::BottomRight => {
                    (max_size.width - size.width, max_size.height - size.height)
                }
            };
            widget.set_origin(ctx, data, env, Point::new(x, y));
        }
        max_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        for widget in self.widgets.iter_mut() {
            widget.paint(ctx, data, env);
        }
    }
}

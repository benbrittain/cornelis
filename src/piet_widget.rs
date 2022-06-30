use crate::AppData;
use druid::{
    BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Rect, RenderContext, Size, UpdateCtx, Widget,
};

pub struct PietViewWidget {
    pub cell_size: Size,
}

impl Widget<AppData> for PietViewWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::WindowConnected => {
                ctx.request_paint();
            }
            _ => (), //  log!("{:?}", &event),
        };
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppData,
        _env: &Env,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppData, data: &AppData, _env: &Env) {
        if data.env != old_data.env {
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        let max_size = bc.max();
        let min_side = max_size.height.min(max_size.width);
        Size {
            width: min_side,
            height: min_side,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, _env: &Env) {
        let size: Size = ctx.size();
        let w0 = (size.width as u32 / data.env.image.png_info.width) as f64;
        let h0 = (size.height as u32 / data.env.image.png_info.height) as f64;
        let cell_size = Size {
            width: w0,
            height: h0,
        };
        self.cell_size = cell_size;
        for col in 0..data.env.image.png_info.width {
            for row in 0..data.env.image.png_info.height {
                let point = Point {
                    x: w0 * col as f64,
                    y: h0 * row as f64,
                };
                let codel = crate::ty::Codel::new(col, row);
                let rect = Rect::from_origin_size(point, cell_size);
                let color_raw = &data.env.image[codel];
                let color = Color::rgb8(color_raw[0], color_raw[1], color_raw[2]);
                ctx.fill(rect, &color);
            }
        }

        let point = Point {
            x: (w0 as u32 * data.env.cp.x) as f64 + (w0 as u32 / 2) as f64,
            y: (h0 as u32 * data.env.cp.y) as f64 + (h0 as u32 / 2) as f64,
        };
        let radius = (w0 + h0) / 2.0;
        let circle = druid::piet::kurbo::Circle::new(point, radius * 0.4);
        ctx.fill(circle, &Color::rgb8(0x48, 0x1e, 0x40));
        let inner_circle = druid::piet::kurbo::Circle::new(point, radius * 0.2);
        ctx.fill(inner_circle, &Color::rgb8(0xff, 0xff, 0xff));
    }
}

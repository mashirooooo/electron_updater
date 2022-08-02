use druid::debug_state::DebugState;
use druid::widget::prelude::*;
use druid::widget::Flex;
use druid::Application;
use druid::{
    theme, AppLauncher, Color, Data, Lens, LinearGradient, Point, Rect, UnitPoint, WidgetExt,
    WindowDesc,
};
#[derive(Clone, Data, Lens, Default)]
struct UpdateState {
    progressbar: f64,
}

pub fn start_ui() {
    // describe the main window
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget())
        .title("更新程序")
        .window_size((400.0, 40.0))
        .resizable(false)
        .show_titlebar(false);
    // create the initial app state
    // 进度条显示
    let initial_state: UpdateState = UpdateState { progressbar: 0.0 };
    // start the application. Here we pass in the application state.
    let launcher = AppLauncher::with_window(main_window);
    // 给进度条的回调
    let event_sink = launcher.get_external_handle();
    std::thread::spawn(move || update(event_sink));
    launcher
        // .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}
// 更新程序及显示ui
fn update(event_sink: druid::ExtEventSink) {
    let quit_app_fn = || {
        event_sink.add_idle_callback(move |_: &mut UpdateState| {
            Application::global().quit();
        })
    };
    let ui_callback = |process: f64| {
        event_sink.add_idle_callback(move |state: &mut UpdateState| {
            state.progressbar = process;
        })
    };
    // 更新
    crate::task::run_task(&quit_app_fn, &ui_callback);
}

fn build_root_widget() -> impl Widget<UpdateState> {
    Flex::column()
        .with_spacer(10.0)
        .with_child(
            ProgressBarWidget::new()
                .on_added(|_, _, _, _| {})
                .lens(UpdateState::progressbar),
        )
        .must_fill_main_axis(true)
        .background(Color::rgb8(0xBA, 0xBA, 0xBA))
}

#[derive(Debug, Clone, Default)]
struct ProgressBarWidget {}
impl ProgressBarWidget {
    pub fn new() -> ProgressBarWidget {
        Self::default()
    }
}
impl Widget<f64> for ProgressBarWidget {
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
        _env: &Env,
    ) -> Size {
        // width height 宽高
        bc.constrain((360.0, 40.0))
    }
    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, env: &Env) {
        let height = env.get(theme::BASIC_WIDGET_HEIGHT);
        let corner_radius = env.get(theme::PROGRESS_BAR_RADIUS);
        let clamped = data.max(0.0).min(1.0);
        let stroke_width = 2.0;
        let inset = -stroke_width / 2.0;
        let size = ctx.size();
        // let str = format!("{:.2}%", clamped * 100.0);
        let rounded_rect = Size::new(size.width, height)
            .to_rect()
            .inset(inset)
            .to_rounded_rect(corner_radius);

        // Paint the border
        ctx.stroke(rounded_rect, &env.get(theme::BORDER_DARK), stroke_width);

        // Paint the background
        let background_gradient = LinearGradient::new(
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
            (
                env.get(theme::BACKGROUND_LIGHT),
                env.get(theme::BACKGROUND_DARK),
            ),
        );
        ctx.fill(rounded_rect, &background_gradient);

        // Paint the bar
        let calculated_bar_width = clamped * rounded_rect.width();

        let rounded_rect = Rect::from_origin_size(
            Point::new(-inset, 0.),
            Size::new(calculated_bar_width, height),
        )
        .inset((0.0, inset))
        .to_rounded_rect(corner_radius);

        let bar_gradient = LinearGradient::new(
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
            (env.get(theme::PRIMARY_LIGHT), env.get(theme::PRIMARY_DARK)),
        );
        ctx.fill(rounded_rect, &bar_gradient);
    }
    fn debug_state(&self, data: &f64) -> DebugState {
        DebugState {
            display_name: self.short_type_name().to_string(),
            main_value: data.to_string(),
            ..Default::default()
        }
    }
}

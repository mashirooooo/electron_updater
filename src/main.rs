// 关闭window子系统
#![windows_subsystem = "windows"]

fn main() {
    #[cfg(feature = "druid")]
    updater::ui::start_ui();
    #[cfg(not(feature = "druid"))]
    let quit_app_fn = || {
        std::process::exit(0);
    };
    #[cfg(not(feature = "druid"))]
    updater::task::run_task(&quit_app_fn);
}

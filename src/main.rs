// 关闭window子系统
#![windows_subsystem = "windows"]
// #[cfg(any(windows))]
// use std::os::windows::prelude::FileExt;
// #[cfg(any(windows))]
// fn read_at(file: &mut File, buffer: &mut [u8], offset: u64) -> Result<usize, std::io::Error> {
//     file.seek_read(buffer, offset)
// }

// #[cfg(any(unix))]
// use std::os::unix::prelude::FileExt;
// #[cfg(any(unix))]
// fn read_at(file: &mut File, buffer: &mut [u8], offset: u64) -> Result<usize, std::io::Error> {
//     file.read_at(buffer, offset)
// }

fn main() {
    if cfg!(target_os = "linux") {
        let current_exe_path = std::env::current_exe().unwrap();
        let is_main = std::env::var("RUST_ELECTRON_MAIN").is_ok();
        if !is_main {
            std::process::Command::new(current_exe_path)
                .env("RUST_ELECTRON_MAIN", "1")
                .envs(std::env::vars())
                .spawn()
                .expect("failed to execute process");
        } else {
            #[cfg(feature = "druid")]
            updater::ui::start_ui();
            #[cfg(not(feature = "druid"))]
            let quit_app_fn = || {
                std::process::exit(0);
            };
            #[cfg(not(feature = "druid"))]
            updater::updater::update(&quit_app_fn);
        }
    } else if cfg!(target_os = "windows") {
        #[cfg(feature = "druid")]
        updater::ui::start_ui();
        #[cfg(not(feature = "druid"))]
        let quit_app_fn = || {
            std::process::exit(0);
        };
        #[cfg(not(feature = "druid"))]
        updater::updater::update(&quit_app_fn);
    } else {
        println!("Unsupported OS");
    }
}

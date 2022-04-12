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
    #[cfg(feature = "druid")]
    updater::ui::start_ui();

    #[cfg(not(feature = "druid"))]
    let quit_app_fn = || {
        std::process::exit(0);
    };
    #[cfg(not(feature = "druid"))]
    updater::updater::update(&quit_app_fn);
}

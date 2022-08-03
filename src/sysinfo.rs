use std::path::Path;
use sysinfo::{ProcessExt, SystemExt};

use crate::mlog::{Log, Logtrait};

/// 结束electron的进程
///
/// # Examples
///
/// ```
/// let path =  Path::new("/usr/bin/electron");
/// let result = end_electron_main(path);
///
/// ```
pub fn end_electron_main<P: AsRef<Path>>(path: P) -> bool {
    let mut sys = sysinfo::System::new_all();
    let current_exe_path = std::env::current_exe().unwrap();
    //注意 如果有其他进程的执行exe的路径是直接kill掉处理
    sys.processes().iter().for_each(|(_pid, process)| {
        match process.exe() {
            v if v.starts_with(path.as_ref())
                && v != current_exe_path
                && process.cmd().iter().any(|x| x == "/prefetch:1") =>
            {
                // Log::info(format!("{process:#?}").as_str());
                process.kill();
            }
            _ => (),
        }
    });
    std::thread::sleep(std::time::Duration::from_millis(50));
    sys.refresh_all();
    sys.processes()
        .iter()
        .for_each(|(_pid, process)| match process.exe() {
            v if v.starts_with(path.as_ref()) && v != current_exe_path => {
                process.kill();
            }
            _ => (),
        });
    std::thread::sleep(std::time::Duration::from_millis(50));
    sys.refresh_all();
    sys.processes().iter().any(|(_pid, process)| {
        let v = process.exe();
        let r = v.starts_with(path.as_ref()) && v != current_exe_path;
        if r {
            Log::error(format!("存在未退出的electron进程: {process:#?}").as_str());
            panic!("存在未退出的electron进程{:?}", process.exe());
        }
        r
    })
}

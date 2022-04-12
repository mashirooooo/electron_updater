use std::path::Path;
use sysinfo::{ProcessExt, Signal, SystemExt};

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
    let sys = sysinfo::System::new_all();
    let current_exe_path = std::env::current_exe().unwrap();
    //注意 如果有其他进程的执行exe的路径是直接kill掉处理
    sys.processes()
        .iter()
        .all(|(_pid, process)| match process.exe() {
            v if v.starts_with(path.as_ref()) && v != current_exe_path => {
                process.kill(Signal::Kill)
            }
            _ => true,
        })
}

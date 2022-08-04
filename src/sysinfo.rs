use std::{env, path::Path};
use sysinfo::{Pid, ProcessExt, SystemExt};

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
    Log::info("尝试结束进程");
    let mut sys = sysinfo::System::new_all();
    match env::var("exe_pid") {
        Ok(pid) if pid.parse::<usize>().is_ok() => {
            if let Some(process) = sys.process(Pid::from(pid.parse::<usize>().unwrap())) {
                process.kill();
            }
        }
        _ => (),
    }
    std::thread::sleep(std::time::Duration::from_millis(50));
    let current_exe_path = std::env::current_exe().unwrap();
    sys.processes()
        .iter()
        .for_each(|(_pid, process)| match process.exe() {
            v if v.starts_with(path.as_ref()) && v != current_exe_path => {
                Log::info(format!("再次尝试结束进程 {v:?} {process:#?}").as_str());
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

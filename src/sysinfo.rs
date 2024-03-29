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
    Log::info("尝试结束进程2");
    let mut sys = sysinfo::System::new_all();
    match env::var("exe_pid") {
        Ok(pid) if pid.parse::<usize>().is_ok() => {
            Log::info(format!("pid进程: {pid:#?}").as_str());

            // []
            #[cfg(any(
                target_os = "freebsd",
                target_os = "linux",
                target_os = "android",
                target_os = "macos",
                target_os = "ios",
            ))]
            let pid = pid.parse::<i32>().unwrap();

            #[cfg(target_os = "windows")]
            let pid = pid.parse::<usize>().unwrap();
            if let Some(process) = sys.process(Pid::from(pid)) {
                process.kill();
            }
        }
        _ => (),
    }
    std::thread::sleep(std::time::Duration::from_millis(50));
    sys.processes()
        .iter()
        .for_each(|(_pid, process)| match process.exe() {
            v if v == path.as_ref() => {
                Log::info(format!("再次尝试结束进程 {v:?} {process:#?}").as_str());
                process.kill();
            }
            _ => (),
        });
    std::thread::sleep(std::time::Duration::from_millis(50));
    sys.refresh_all();
    sys.processes().iter().any(|(_pid, process)| {
        let v = process.exe();
        let r = v == path.as_ref();
        if r {
            Log::error(format!("存在未退出的electron进程: {process:#?}").as_str());
            panic!("存在未退出的electron进程{:?}", process.exe());
        }
        r
    })
}

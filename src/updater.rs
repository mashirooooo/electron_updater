use std::{env, fs, path::Path, process, thread, time::Duration};

use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::sysinfo::end_electron_main;

// todo 出错后版本后退问题， 中断继续问题
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct FileHashAndPath {
    filePath: String,
    hash: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct UpdateConfigJson {
    added: Vec<FileHashAndPath>,
    changed: Vec<FileHashAndPath>,
}

fn copy_file<P: AsRef<Path>>(
    config: &UpdateConfigJson,
    path: P,
    update_temp_path: P,
    #[cfg(feature = "druid")] ui_callback: &(dyn Fn(f64)),
) {
    #[cfg(feature = "druid")]
    let mut hand_file_num = 0.0;
    #[cfg(feature = "druid")]
    let total_file = (config.added.len() + config.changed.len()) as f64;
    // 结束进程后迁移文件
    for item in config.added.iter().chain(config.changed.iter()) {
        hand_file_num += 1.0;
        let file_path = path.as_ref().join(&item.filePath);
        // println!("{:?} {}", &file_path, hand_file_num);
        let from_path = Path::new(update_temp_path.as_ref()).join(&item.hash);

        #[cfg(feature = "druid")]
        ui_callback((hand_file_num / total_file) as f64);

        // ui绘制时间
        thread::sleep(Duration::from_millis(10));

        if !from_path.is_file() {
            // 缺少依赖处理 todo
            continue;
        }
        if file_path.is_file() && fs::remove_file(&file_path).is_err() {
            // 删除对应文件产生错误处理 todo
        }
        if fs::create_dir_all(file_path.parent().unwrap()).is_err() {
            // 创建父文件夹失败处理 todo
        }
        if fs::copy(from_path, file_path).is_err() {
            // 复制文件到对应路径错误处理 todo
        }
    }
}
/// 更新程序
pub fn update(quit_app_fn: &(dyn Fn()), #[cfg(feature = "druid")] ui_callback: &(dyn Fn(f64))) {
    // 当前执行exe的 没传过来直接结束进程
    std::thread::sleep(std::time::Duration::from_millis(100));
    let exe_path_buf = match env::var("exe_path") {
        Ok(path) if Path::new(&path).is_absolute() => Path::new(&path).to_owned(),
        _ => {
            quit_app_fn();
            return;
        }
    };
    let exe_path = exe_path_buf.as_path();
    let path = exe_path.parent().unwrap();
    // println!("{:?}", path);
    // 更新temp目录
    let update_temp_path = match env::var("update_temp_path") {
        Ok(path) if Path::new(&path).is_absolute() => Path::new(&path).to_owned(),
        _ => Path::new(&path).join("update_temp"),
    };
    // println!("{:?}", update_temp_path);
    let update_config_file_name = match env::var("update_config_file_name") {
        Ok(name) => name,
        _ => "update-config.json".to_string(),
    };
    // println!("{:?}", update_config_file_name);
    // println!("{:?}", update_temp_path.join(&update_config_file_name));
    // todo 处理读取更新配置出错
    let config: UpdateConfigJson = match serde_json::from_slice(
        &fs::read(update_temp_path.join(update_config_file_name)).unwrap_or_default(),
    ) {
        Ok(config) => config,
        _ => {
            // 退出程序
            quit_app_fn();
            return;
        }
    };
    // 处理未关闭的exe进程
    if !config.added.is_empty() || !config.changed.is_empty() {
        end_electron_main(&path);
    }
    // 复制文件
    copy_file(
        &config,
        &path,
        &update_temp_path.as_path(),
        #[cfg(feature = "druid")]
        ui_callback,
    );
    // 重启程序
    process::Command::new(exe_path).spawn().unwrap();
    // 退出更新程序
    quit_app_fn();
}

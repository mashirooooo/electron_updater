use std::{env, fs, path::Path, process, thread, time::Duration};

use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::{
    mlog::{Log, Logtrait},
    sysinfo::end_electron_main,
};

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
    let mut hand_file_num = 0.0;
    #[cfg(feature = "druid")]
    let total_file = (config.added.len() + config.changed.len()) as f64;
    Log::info("总共需要迁移得文件为");
    Log::info(total_file.to_string().as_str());
    // 结束进程后迁移文件
    for item in config.added.iter().chain(config.changed.iter()) {
        hand_file_num += 1.0;
        let file_path = path.as_ref().join(&item.filePath);
        Log::info("迁移的目标文件:");
        Log::info(file_path.to_str().unwrap());

        let from_path = Path::new(update_temp_path.as_ref()).join(&item.hash);
        Log::info("迁移的源文件:");
        Log::info(from_path.to_str().unwrap());
        #[cfg(feature = "druid")]
        ui_callback((hand_file_num / total_file) as f64);

        // ui绘制时间
        thread::sleep(Duration::from_millis(10));

        if !from_path.is_file() {
            Log::error("迁移的源文件不是文件file, 跳过该项");
            // 缺少依赖处理
            continue;
        }
        if file_path.is_file() && fs::remove_file(&file_path).is_err() {
            Log::error("删除旧目标文件产生错误");
            panic!("删除旧目标文件产生错误");
            // 删除对应文件产生错误处理
        }
        if fs::create_dir_all(file_path.parent().unwrap()).is_err() {
            Log::error("创建目标父文件夹失败");
            panic!("创建目标父文件夹失败");
            // 创建父文件夹失败处理
        }
        if fs::copy(from_path, file_path).is_err() {
            Log::error("复制源文件到对应路径错误");
            panic!("复制源文件到对应路径错误");
            // 复制文件到对应路径错误处理
        }
    }
}
/// 更新程序
pub fn update(quit_app_fn: &(dyn Fn()), #[cfg(feature = "druid")] ui_callback: &(dyn Fn(f64))) {
    Log::info("程序开始");
    // 当前执行exe的 没传过来直接结束进程
    std::thread::sleep(std::time::Duration::from_millis(100));
    let exe_path_buf = match env::var("exe_path") {
        Ok(path) if Path::new(&path).is_absolute() => Path::new(&path).to_owned(),
        _ => {
            Log::error("获取exe_path变量错误; 程序将退出");
            quit_app_fn();
            return;
        }
    };
    let exe_path = exe_path_buf.as_path();
    Log::info("exe_path路径: ");
    Log::info(exe_path.to_str().unwrap());
    let path = exe_path.parent().unwrap();
    Log::info("根目录: ");
    Log::info(path.to_str().unwrap());
    // 更新temp目录
    let update_temp_path = match env::var("update_temp_path") {
        Ok(path) if Path::new(&path).is_absolute() => Path::new(&path).to_owned(),
        _ => Path::new(&path).join("update_temp"),
    };
    Log::info("更新temp目录: ");
    Log::info(update_temp_path.to_str().unwrap());

    let update_config_file_name = match env::var("update_config_file_name") {
        Ok(name) => name,
        _ => "update-config.json".to_string(),
    };
    Log::info("配置update_config_file_name: ");
    Log::info(update_config_file_name.as_str());
    // println!("{:?}", update_config_file_name);
    // println!("{:?}", update_temp_path.join(&update_config_file_name));
    // todo 处理读取更新配置出错
    Log::info("读取更新配置：");
    Log::info("读取更新配置路径：");
    Log::info(
        update_temp_path
            .join(&update_config_file_name)
            .to_str()
            .unwrap(),
    );
    let config: UpdateConfigJson = match serde_json::from_slice(
        &fs::read(update_temp_path.join(update_config_file_name)).unwrap_or_default(),
    ) {
        Ok(config) => config,
        _ => {
            Log::error("读取更新配置失败：");
            // 退出程序
            quit_app_fn();
            return;
        }
    };
    Log::info("读取更新配置为：");
    Log::info(format!("{config:#?}").as_str());
    Log::info("开始更新");
    Log::info("处理未关闭的electron进程");
    // 处理未关闭的exe进程
    if !config.added.is_empty() || !config.changed.is_empty() {
        end_electron_main(&path);
    }
    Log::info("迁移文件");
    // 复制文件
    copy_file(
        &config,
        &path,
        &update_temp_path.as_path(),
        #[cfg(feature = "druid")]
        ui_callback,
    );
    Log::info("迁移文件结束，更新完成");
    Log::info("重启程序");
    // 重启程序
    process::Command::new(exe_path).spawn().unwrap();
    Log::info("退出更新程序");
    // 退出更新程序
    quit_app_fn();
}

use std::{
    collections::HashMap,
    env, fs,
    io::{Seek, Write},
    path::{Path, PathBuf},
    process, thread,
    time::Duration,
};

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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RunningState {
    Nothing = 0,
    Updating,
    UpdateButNotCheck,
    Finish,
    Failed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RunningConfig {
    pub status: RunningState,
    pub file_path: HashMap<usize, String>,
    pub update_temp_path: String,
    pub exe_path: String,
    pub moved_path: Vec<String>, //path
}

fn check_permission<P: AsRef<Path>>(
    config: &UpdateConfigJson,
    path: P,
    update_temp_path: P,
    running_config: &mut RunningConfig,
) -> bool {
    let update_temp_path_old_p =
        Path::new(update_temp_path.as_ref()).join(".update_temp_path_old_version");
    if update_temp_path_old_p.exists() {
        if let Err(e) = fs::remove_dir_all(&update_temp_path_old_p) {
            Log::error("清除更新缓存文件夹读取：");
            Log::error(e.to_string().as_str());
            return false;
        }
    }
    // 创建文件夹
    fs::create_dir_all(&update_temp_path_old_p).unwrap();
    running_config.file_path = HashMap::new();
    let mut move_target = Vec::new();
    for (index, item) in config.added.iter().chain(config.changed.iter()).enumerate() {
        let file_path = path.as_ref().join(&item.filePath);
        let from_path = Path::new(update_temp_path.as_ref()).join(&item.hash);
        let to_path = update_temp_path_old_p.join(index.to_string());
        let check = {
            if !from_path.is_file() {
                Log::error("缺少迁移的目标文件:");
                Log::error(from_path.to_str().unwrap());
                false
            } else if fs::create_dir_all(file_path.parent().unwrap()).is_err() {
                Log::error("创建目标父文件夹失败");
                Log::error(file_path.parent().unwrap().to_str().unwrap());
                false
                // 创建父文件夹失败处理
            } else if file_path.exists() {
                if fs::rename(&file_path, &to_path).is_ok() {
                    running_config
                        .file_path
                        .insert(index, file_path.to_str().unwrap().to_owned());
                    move_target.push((file_path, to_path));
                    true
                } else {
                    Log::error("rename文件失败");
                    Log::error(file_path.to_str().unwrap());
                    false
                }
            } else {
                true
            }
        };
        if !check {
            // 回退
            for (from, to) in move_target.iter() {
                fs::rename(to, from).unwrap();
            }
            running_config.file_path.clear();
            return false;
        }
    }

    true
}

fn copy_file<P: AsRef<Path>>(
    config: &UpdateConfigJson,
    path: P,
    update_temp_path: P,
    running_config_file: &mut fs::File,
    running_config: &mut RunningConfig,
    #[cfg(feature = "druid")] ui_callback: impl Fn(f64),
) -> bool {
    let mut hand_file_num = 0.0;
    let total_file = (config.added.len() + config.changed.len()) as f64;
    Log::info("总共需要迁移得文件为");
    Log::info(total_file.to_string().as_str());
    // 结束进程后迁移文件
    for item in config.added.iter().chain(config.changed.iter()) {
        hand_file_num += 1.0;
        Log::info(format!(" 当前迁移第{}个文件", hand_file_num as u32).as_str());
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

        if fs::copy(from_path, &file_path).is_err() {
            Log::error("复制源文件到对应路径错误");
            Log::error(file_path.to_str().unwrap());
            running_config.status = RunningState::Failed;
            flush_config_file(running_config_file, running_config);
            // 复制文件到对应路径错误处理
            return false;
        } else {
            running_config
                .moved_path
                .push(file_path.to_str().unwrap().to_owned());
            flush_config_file(running_config_file, running_config);
        }
    }
    true
}

// 刷新config文件
fn flush_config_file(running_config_file: &mut fs::File, running_config: &RunningConfig) {
    running_config_file.set_len(0).unwrap();
    running_config_file.rewind().unwrap();
    running_config_file
        .write_all(serde_json::to_string(running_config).unwrap().as_bytes())
        .unwrap();
    running_config_file.sync_all().unwrap();
}

// 更新
fn update(
    quit_app_fn: impl Fn(),
    exe_path_buf: PathBuf,
    skip_check: bool,
    mut running_config: RunningConfig,
    runnning_config_path: &Path,
    #[cfg(feature = "druid")] ui_callback: impl Fn(f64),
) {
    let mut running_config_file = {
        if runnning_config_path.exists() {
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(runnning_config_path)
                .unwrap()
        } else {
            fs::File::create(runnning_config_path).unwrap()
        }
    };
    running_config.status = RunningState::Updating;
    flush_config_file(&mut running_config_file, &running_config);
    // running_config_file.write_all(buf)
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
    running_config.update_temp_path = update_temp_path
        .join(&update_config_file_name)
        .to_str()
        .unwrap()
        .to_owned();
    flush_config_file(&mut running_config_file, &running_config);
    let config: UpdateConfigJson = match serde_json::from_slice(
        &fs::read(update_temp_path.join(update_config_file_name)).unwrap_or_default(),
    ) {
        Ok(config) => config,
        _ => {
            Log::error("读取更新配置失败：");
            // 重置running状态
            running_config.status = RunningState::Nothing;
            flush_config_file(&mut running_config_file, &running_config);
            // 退出程序
            quit_app_fn();
            return;
        }
    };
    Log::info("读取更新配置为：");
    Log::info(format!("{config:#?}").as_str());
    Log::info("开始更新");
    Log::info("处理未关闭的electron进程");
    if !skip_check {
        {
            if !check_permission(
                &config,
                path,
                update_temp_path.as_path(),
                &mut running_config,
            ) {
                // 检测权限不通过，更新结束
                running_config.status = RunningState::Nothing;
                flush_config_file(&mut running_config_file, &running_config);
                Log::error("检测权限不通过，更新结束");
                quit_app_fn();
                return;
            };
        }
        running_config.status = RunningState::Updating;
        flush_config_file(&mut running_config_file, &running_config);
    }

    // 处理未关闭的exe进程
    if !config.added.is_empty() || !config.changed.is_empty() {
        end_electron_main(&path);
    }
    Log::info("迁移文件");
    // 复制文件
    if !copy_file(
        &config,
        &path,
        &update_temp_path.as_path(),
        &mut running_config_file,
        &mut running_config,
        #[cfg(feature = "druid")]
        ui_callback,
    ) {
        callback(&mut running_config_file, &mut running_config);
    } else {
        // 处理更新文件
        running_config.status = RunningState::Finish;
        flush_config_file(&mut running_config_file, &running_config);
        Log::info("迁移文件结束，更新完成");
        Log::info("清理更新文件");
        if let Err(e) = fs::remove_dir_all(update_temp_path) {
            Log::error("清理更新文件出错：");
            Log::error(e.to_string().as_str());
        }
        Log::info("清理更新文件完成");
        Log::info("重启程序");
        // 重启程序
        process::Command::new(exe_path)
            .env("updateCallback", "success")
            .spawn()
            .unwrap();
        Log::info("退出更新程序");
        // 退出更新程序
        quit_app_fn();
    }
}

fn callback(running_config_file: &mut fs::File, running_config: &mut RunningConfig) {
    let update_temp_path = Path::new(&running_config.update_temp_path);
    running_config.status = RunningState::Failed;
    flush_config_file(running_config_file, running_config);

    let update_temp_path_old_p = Path::new(update_temp_path).join(".update_temp_path_old_version");

    // delete moved file
    running_config.moved_path.iter().for_each(|i| {
        let p = Path::new(i);
        fs::remove_file(p).unwrap();
    });
    running_config.moved_path.clear();
    flush_config_file(running_config_file, running_config);

    // 还原
    running_config.file_path.iter().for_each(|(index, from)| {
        let to_path = update_temp_path_old_p.join(index.to_string());
        let from_path = PathBuf::from(from);
        fs::rename(to_path, from_path).unwrap();
    });
    running_config.status = RunningState::Nothing;
    running_config.file_path.clear();
    flush_config_file(running_config_file, running_config);
}

/// 执行的程序
pub fn run_task(quit_app_fn: impl Fn(), #[cfg(feature = "druid")] ui_callback: impl Fn(f64)) {
    Log::info("程序开始");
    // 当前执行exe的 没传过来直接结束进程
    std::thread::sleep(std::time::Duration::from_millis(100));
    Log::info("获取electron程序的执行目录,判断任务状态");
    let running_config_path = Path::new(".running_status");

    match env::var("exe_path") {
        Ok(path) if Path::new(&path).is_absolute() => {
            Log::info("执行更新程序");
            let exe_path_buf = Path::new(&path).to_owned();
            let config = RunningConfig {
                status: RunningState::UpdateButNotCheck,
                file_path: HashMap::new(),
                exe_path: exe_path_buf.to_str().unwrap().to_string(),
                update_temp_path: String::new(),
                moved_path: Vec::new(),
            };
            update(
                quit_app_fn,
                exe_path_buf,
                false,
                config,
                running_config_path,
                ui_callback,
            );
        }
        _ => {
            // 判断是回滚还是继续更新
            Log::error("获取exe_path变量错误; 程序将退出");
            if !running_config_path.exists() {
                // 不存在运行状态，无任务，直接退出
                Log::info("程序无执行任务");
                quit_app_fn();
            } else {
                // 读取运行状态
                match serde_json::from_slice::<RunningConfig>(
                    &fs::read(running_config_path).unwrap_or_default(),
                ) {
                    Ok(mut config) if config.status == RunningState::Failed => {
                        let mut running_config_file = {
                            if running_config_path.exists() {
                                fs::OpenOptions::new()
                                    .read(true)
                                    .write(true)
                                    .open(running_config_path)
                                    .unwrap()
                            } else {
                                fs::File::create(running_config_path).unwrap()
                            }
                        };
                        // 更新失败 回滚数据
                        callback(&mut running_config_file, &mut config);
                        quit_app_fn();
                    }
                    Ok(config) if config.status == RunningState::Updating => {
                        // 继续更新
                        let exe_path_buf = PathBuf::from(&config.exe_path);
                        update(
                            quit_app_fn,
                            exe_path_buf,
                            true,
                            config,
                            running_config_path,
                            ui_callback,
                        );
                    }
                    Ok(config) if config.status == RunningState::UpdateButNotCheck => {
                        // 继续更新
                        let exe_path_buf = PathBuf::from(&config.exe_path);
                        update(
                            quit_app_fn,
                            exe_path_buf,
                            false,
                            config,
                            running_config_path,
                            ui_callback,
                        );
                    }
                    Ok(_) => {
                        Log::info("程序无执行任务");
                        // 退出程序
                        quit_app_fn();
                    }
                    _ => {
                        Log::error("读取运行配置失败：");
                        // 退出程序
                        quit_app_fn();
                    }
                };
            }
        }
    };
}

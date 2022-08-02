use super::Logtrait;
use log::{debug, error, info, warn};
pub struct Log {}
impl Logtrait for Log {
    fn setup_logging() {
        let base_config = fern::Dispatch::new().level(log::LevelFilter::Debug);
        let file_config = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .chain(fern::DateBased::new("log.", "%Y-%m-%d"));
        base_config.chain(file_config).apply().unwrap();
    }
    fn info(info: &str) {
        info!("{info}");
    }
    fn debug(debug: &str) {
        debug!("{debug}");
    }

    fn warn(warn: &str) {
        warn!("{warn}");
    }

    fn error(error: &str) {
        error!("{error}");
    }
}

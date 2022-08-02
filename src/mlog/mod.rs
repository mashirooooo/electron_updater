pub trait Logtrait {
    fn setup_logging() {}
    fn info(_info: &str) {}
    fn debug(_debug: &str) {}
    fn warn(_warn: &str) {}
    fn error(_error: &str) {}
}
#[cfg(feature = "debug")]
mod log_debug;

#[cfg(feature = "debug")]
pub use log_debug::Log;

#[cfg(not(feature = "debug"))]
pub struct Log {}
#[cfg(not(feature = "debug"))]
impl Logtrait for Log {}

#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;

use fast_log;
use log::{error, info};

use crate::LOG_LOCATION;

/// TODO Needs to make check if I have access to folder before I write db.
pub fn main() {
    let log_bool = Path::new(LOG_LOCATION).exists();

    if log_bool {
        fs::remove_file(LOG_LOCATION).unwrap();
    }

    fast_log::init(fast_log::Config::new().file(LOG_LOCATION)).unwrap();
    info!("Initing Logger.");
    log::logger().flush();
}

///
/// Dumps error to log and panics.
///
pub fn panic_log(error: &String) {
    error!("{}", error);
    panic!("{}", error);
}
///
/// Dumps error to log and panics.
///
pub fn error_log(error: &String) {
    println!("{}", error);
    error!("{}", error);
}

///
/// Dumps info to log and prints it.
///
pub fn info_log(info: &String) {
    info!("{}", info);
    println!("{}", info);
}

///
/// Dumps info to log and prints it.
///
pub fn log(info: &String) {
    info!("{}", info);
}

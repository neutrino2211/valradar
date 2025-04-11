use std::env;

/// Print a debug message if debug mode is enabled
pub fn debug(message: &str) -> () {
    if env::var("VALRADAR_DEBUG").is_ok() {
        println!("{}", message);
    }
} 
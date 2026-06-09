#[macro_export]
macro_rules! log_info {
    ($component:expr, $($arg:tt)*) => {
        println!("[{}] [{}] [INFO] {}",
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            $component,
            format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($component:expr, $($arg:tt)*) => {
        eprintln!("[{}] [{}] [ERROR] {}",
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            $component,
            format!($($arg)*))
    };
}

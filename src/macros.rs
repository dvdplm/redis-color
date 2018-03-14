macro_rules! error {
    ($message:expr) => {
        ColorError::generic($message)
    };
    ($message:expr, $($arg:tt)*) => {
        ColorError::generic(format!($message, $($arg)+).as_str())
    }
}
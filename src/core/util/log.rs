//! ロギングユーティリティ

/// ログレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// ログ出力（シリアルとVGAの両方）
pub fn log(level: LogLevel, args: core::fmt::Arguments) {
    use crate::{sprint, sprintln, vprint, vprintln};

    let prefix = match level {
        LogLevel::Trace => "[TRACE]",
        LogLevel::Debug => "[DEBUG]",
        LogLevel::Info => "[INFO] ",
        LogLevel::Warn => "[WARN] ",
        LogLevel::Error => "[ERROR]",
    };

    sprint!("{} ", prefix);
    sprintln!("{}", args);
}

/// トレースログ
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::util::log::log($crate::util::log::LogLevel::Trace, format_args!($($arg)*))
    };
}

/// デバッグログ
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::util::log::log($crate::util::log::LogLevel::Debug, format_args!($($arg)*))
    };
}

/// 情報ログ
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::util::log::log($crate::util::log::LogLevel::Info, format_args!($($arg)*))
    };
}

/// 警告ログ
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::util::log::log($crate::util::log::LogLevel::Warn, format_args!($($arg)*))
    };
}

/// エラーログ
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::util::log::log($crate::util::log::LogLevel::Error, format_args!($($arg)*))
    };
}

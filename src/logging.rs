//! # 日志系统模块
//!
//! 提供线程安全的日志记录功能，支持多个日志级别（Trace/Debug/Info/Warn/Error），
//! 同时输出到控制台和日志文件。

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use time::macros::format_description;
use time::OffsetDateTime;

use super::config::{LogLevel, LoggingConfig};

/// 日志记录器结构体
///
/// 支持多级别日志输出，线程安全，可同时输出到控制台和文件。
#[allow(dead_code)]
pub struct Logger {
    level: LogLevel,
    file_writer: Option<Arc<Mutex<BufWriter<File>>>>,
    max_file_size_mb: u64,
    max_backups: usize,
    current_size: Arc<Mutex<u64>>,
}

#[allow(dead_code)]
impl Logger {
    /// 创建日志记录器实例
    ///
    /// # 参数
    /// - `config`: 日志配置
    ///
    /// # 返回值
    /// 成功返回 Logger 实例，失败返回 IO 错误
    pub fn new(config: &LoggingConfig) -> Result<Self, std::io::Error> {
        let file_writer = if let Some(file_path) = config.get_file_path() {
            let path = Path::new(file_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            let file = File::options().create(true).append(true).open(path)?;

            let _metadata = file.metadata()?;

            Some(Arc::new(Mutex::new(BufWriter::new(file))))
        } else {
            None
        };

        Ok(Logger {
            level: config.get_level(),
            file_writer,
            max_file_size_mb: config.get_max_file_size_mb(),
            max_backups: config.get_max_backups(),
            current_size: Arc::new(Mutex::new(0u64)),
        })
    }

    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    fn log(&self, level: LogLevel, message: &str) {
        if !self.should_log(&level) {
            return;
        }

        let timestamp = Self::get_timestamp();
        let log_line = format!("[{}] [{}] {}\n", timestamp, level.as_str(), message);

        println!("{}", log_line.trim());

        if let Some(writer) = &self.file_writer {
            self.write_to_file(writer, &log_line);
        }
    }

    fn should_log(&self, level: &LogLevel) -> bool {
        match self.level {
            LogLevel::Trace => true,
            LogLevel::Debug => *level >= LogLevel::Debug,
            LogLevel::Info => *level >= LogLevel::Info,
            LogLevel::Warn => *level >= LogLevel::Warn,
            LogLevel::Error => *level >= LogLevel::Error,
        }
    }

    fn write_to_file(&self, writer: &Arc<Mutex<BufWriter<File>>>, line: &str) {
        let mut guard = writer.lock().unwrap();
        if let Err(e) = guard.write_all(line.as_bytes()) {
            eprintln!("Failed to write to log file: {}", e);
        }
        if let Err(e) = guard.flush() {
            eprintln!("Failed to flush log file: {}", e);
        }
    }

    fn get_timestamp() -> String {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let seconds = since_epoch.as_secs();
        let nanos = since_epoch.subsec_nanos();

        let time = OffsetDateTime::from_unix_timestamp(seconds as i64)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());

        let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        let formatted_time = time.format(&format).unwrap_or_default();

        format!("{}:{:09}", formatted_time, nanos)
    }

    pub fn log_request(&self, language: &str, success: bool, duration_ms: u64) {
        let status = if success { "SUCCESS" } else { "FAILED" };
        self.info(&format!(
            "Request: language={}, status={}, duration={}ms",
            language, status, duration_ms
        ));
    }

    pub fn log_server_start(&self, host: &str, port: u16) {
        self.info(&format!("Server started on http://{}:{}", host, port));
    }

    pub fn log_error(&self, error: &str, context: &str) {
        self.error(&format!("[{}] Error: {}", context, error));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_order() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("trace"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("DEBUG"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("WARN"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_logger_creation() {
        let config = LoggingConfig::default();
        let logger = Logger::new(&config);
        assert!(logger.is_ok());
    }
}

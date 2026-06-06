//配置管理模块
//负责加载和管理应用程序配置，支持从 config.toml 文件加载配置或使用默认配置。
//包含服务器、编译器和日志三个配置部分。

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 应用程序主配置结构体
#[derive(Debug, Clone)]
pub struct AppConfig {
    server: ServerConfig,
    compiler: CompilerConfig,
    logging: LoggingConfig,
}

/// 服务器配置结构体
#[derive(Debug, Clone)]
pub struct ServerConfig {
    host: String,            // 绑定的主机地址
    port: u16,               // 监听端口
    max_workers: usize,      // 最大工作线程数
    timeout_secs: u64,       // 请求超时时间（秒）
    max_request_size: usize, // 最大请求大小（字节）
}

/// 编译器配置结构体
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    python_path: String,        // Python 解释器路径
    java_path: String,          // Java 编译器路径
    gcc_path: String,           // GCC 编译器路径
    gpp_path: String,           // G++ 编译器路径
    rustc_path: String,         // Rust 编译器路径
    max_compile_time_secs: u64, // 最大编译时间（秒）
    max_run_time_secs: u64,     // 最大运行时间（秒）
    temp_dir: String,           // 临时文件目录
}

/// 日志配置结构体
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    level: LogLevel,           // 日志级别
    file_path: Option<String>, // 日志文件路径（可选）
    max_file_size_mb: u64,     // 单个日志文件最大大小（MB）
    max_backups: usize,        // 日志文件备份数量
}

/// 日志级别枚举
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Copy)]
pub enum LogLevel {
    Trace, // 最详细的日志级别，用于调试
    Debug, // 调试信息
    Info,  // 一般信息
    Warn,  // 警告信息
    Error, // 错误信息
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

impl AppConfig {
    pub fn new() -> Self {
        AppConfig {
            server: ServerConfig::default(),
            compiler: CompilerConfig::default(),
            logging: LoggingConfig::default(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(ConfigError::FileNotFound(
                path.to_string_lossy().to_string(),
            ));
        }

        let content = fs::read_to_string(path)?;
        let config: HashMap<String, String> = parse_config_file(&content)?;

        let mut app_config = AppConfig::new();
        app_config.parse_server_config(&config)?;
        app_config.parse_compiler_config(&config)?;
        app_config.parse_logging_config(&config)?;

        Ok(app_config)
    }

    fn parse_server_config(&mut self, config: &HashMap<String, String>) -> Result<(), ConfigError> {
        if let Some(host) = config.get("server.host") {
            self.server.host = host.clone();
        }
        if let Some(port) = config.get("server.port") {
            self.server.port = port.parse()?;
        }
        if let Some(max_workers) = config.get("server.max_workers") {
            self.server.max_workers = max_workers.parse()?;
        }
        if let Some(timeout) = config.get("server.timeout_secs") {
            self.server.timeout_secs = timeout.parse()?;
        }
        if let Some(max_size) = config.get("server.max_request_size") {
            self.server.max_request_size = max_size.parse()?;
        }
        Ok(())
    }

    fn parse_compiler_config(
        &mut self,
        config: &HashMap<String, String>,
    ) -> Result<(), ConfigError> {
        if let Some(path) = config.get("compiler.python_path") {
            self.compiler.python_path = path.clone();
        }
        if let Some(path) = config.get("compiler.java_path") {
            self.compiler.java_path = path.clone();
        }
        if let Some(path) = config.get("compiler.gcc_path") {
            self.compiler.gcc_path = path.clone();
        }
        if let Some(path) = config.get("compiler.gpp_path") {
            self.compiler.gpp_path = path.clone();
        }
        if let Some(path) = config.get("compiler.rustc_path") {
            self.compiler.rustc_path = path.clone();
        }
        if let Some(time) = config.get("compiler.max_compile_time_secs") {
            self.compiler.max_compile_time_secs = time.parse()?;
        }
        if let Some(time) = config.get("compiler.max_run_time_secs") {
            self.compiler.max_run_time_secs = time.parse()?;
        }
        if let Some(dir) = config.get("compiler.temp_dir") {
            self.compiler.temp_dir = dir.clone();
        }
        Ok(())
    }

    fn parse_logging_config(
        &mut self,
        config: &HashMap<String, String>,
    ) -> Result<(), ConfigError> {
        if let Some(level) = config.get("logging.level") {
            self.logging.level = LogLevel::from_str(level).ok_or_else(|| {
                ConfigError::InvalidValue("logging.level".to_string(), level.clone())
            })?;
        }
        if let Some(path) = config.get("logging.file_path") {
            self.logging.file_path = Some(path.clone());
        }
        if let Some(size) = config.get("logging.max_file_size_mb") {
            self.logging.max_file_size_mb = size.parse()?;
        }
        if let Some(backups) = config.get("logging.max_backups") {
            self.logging.max_backups = backups.parse()?;
        }
        Ok(())
    }

    pub fn get_server_config(&self) -> &ServerConfig {
        &self.server
    }

    #[allow(dead_code)]
    pub fn get_compiler_config(&self) -> &CompilerConfig {
        &self.compiler
    }

    #[allow(dead_code)]
    pub fn get_logging_config(&self) -> &LoggingConfig {
        &self.logging
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue(
                "server.port".to_string(),
                self.server.port.to_string(),
            ));
        }
        if self.server.max_workers == 0 {
            return Err(ConfigError::InvalidValue(
                "server.max_workers".to_string(),
                "0".to_string(),
            ));
        }
        Ok(())
    }
}

impl ServerConfig {
    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_max_workers(&self) -> usize {
        self.max_workers
    }

    pub fn get_timeout_secs(&self) -> u64 {
        self.timeout_secs
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            max_workers: 10,
            timeout_secs: 30,
            max_request_size: 1024 * 1024,
        }
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        CompilerConfig {
            python_path: "python".to_string(),
            java_path: "javac".to_string(),
            gcc_path: "gcc".to_string(),
            gpp_path: "g++".to_string(),
            rustc_path: "rustc".to_string(),
            max_compile_time_secs: 60,
            max_run_time_secs: 30,
            temp_dir: "".to_string(),
        }
    }
}

impl LoggingConfig {
    #[allow(dead_code)]
    pub fn get_level(&self) -> LogLevel {
        self.level
    }

    #[allow(dead_code)]
    pub fn get_file_path(&self) -> &Option<String> {
        &self.file_path
    }

    #[allow(dead_code)]
    pub fn get_max_file_size_mb(&self) -> u64 {
        self.max_file_size_mb
    }

    #[allow(dead_code)]
    pub fn get_max_backups(&self) -> usize {
        self.max_backups
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: LogLevel::Info,
            file_path: None,
            max_file_size_mb: 10,
            max_backups: 5,
        }
    }
}

fn parse_config_file(content: &str) -> Result<HashMap<String, String>, ConfigError> {
    let mut config = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(ConfigError::ParseError(line_num + 1, line.to_string()));
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        config.insert(key.to_string(), value.to_string());
    }

    Ok(config)
}

#[derive(Debug, Clone)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(usize, String),
    InvalidValue(String, String),
    IoError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "Config file not found: {}", path),
            ConfigError::ParseError(line, content) => {
                write!(f, "Parse error at line {}: {}", line, content)
            }
            ConfigError::InvalidValue(key, value) => {
                write!(f, "Invalid value for {}: {}", key, value)
            }
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::IoError(e.to_string())
    }
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(e: std::num::ParseIntError) -> Self {
        ConfigError::InvalidValue("number".to_string(), e.to_string())
    }
}

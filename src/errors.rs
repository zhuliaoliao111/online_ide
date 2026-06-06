//! # 错误处理模块
//! 
//! 定义项目中使用的统一错误类型和错误处理工具。

/// 代码执行相关的错误类型枚举
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum CodeError {
    LanguageNotSupported(String), // 不支持的编程语言
    CompilationError(String),     // 编译错误
    RuntimeError(String),         // 运行时错误
    FileError(String),            // 文件操作错误
    InvalidRequest(String),       // 无效请求
    Timeout,                     // 执行超时
    InternalError,               // 内部服务器错误
}

impl std::fmt::Display for CodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeError::LanguageNotSupported(lang) => write!(f, "Language not supported: {}", lang),
            CodeError::CompilationError(msg) => write!(f, "Compilation failed: {}", msg),
            CodeError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            CodeError::FileError(msg) => write!(f, "File operation failed: {}", msg),
            CodeError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            CodeError::Timeout => write!(f, "Execution timeout"),
            CodeError::InternalError => write!(f, "Internal server error"),
        }
    }
}

impl From<std::io::Error> for CodeError {
    fn from(e: std::io::Error) -> Self {
        CodeError::FileError(e.to_string())
    }
}

/// 项目统一的 Result 类型别名
pub type Result<T> = std::result::Result<T, CodeError>;

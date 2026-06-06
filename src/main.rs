//! # Online Code IDE Server
//! 
//! 在线代码 IDE 后端服务的主入口文件。
//! 负责加载配置、初始化日志系统、启动 HTTP 服务器。

// 导入项目模块
mod analyzer;
mod compiler;
mod config;
mod errors;
mod logging;
mod models;
mod server;

// 导入所需类型
use config::AppConfig;
use logging::Logger;
use server::{CodeServer, ServerConfig};

/// 程序主入口函数
fn main() {
    println!("=== Online Code IDE Server ===");
    println!("Loading configuration...");

    // 加载配置文件，失败时使用默认配置
    let app_config = match AppConfig::load_from_file("config.toml") {
        Ok(config) => {
            println!("Configuration loaded from config.toml");
            config
        }
        Err(e) => {
            println!("Using default configuration: {}", e);
            AppConfig::new()
        }
    };

    // 验证配置有效性
    if let Err(e) = app_config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    // 创建日志记录器
    let logger = match Logger::new(app_config.get_logging_config()) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to create logger: {}", e);
            Logger::new(&config::LoggingConfig::default()).expect("Failed to create default logger")
        }
    };

    // 记录启动信息
    logger.info("=== Online Code IDE Server ===");
    logger.info("Configuration loaded successfully");

    // 创建服务器配置
    let server_config = ServerConfig::new(
        app_config.get_server_config().get_host(),
        app_config.get_server_config().get_port(),
    )
    .with_max_workers(app_config.get_server_config().get_max_workers())
    .with_timeout(app_config.get_server_config().get_timeout_secs());

    // 创建并配置服务器
    let server = CodeServer::new(server_config).with_logger(app_config.get_logging_config());

    logger.info("Starting server...");

    // 启动服务器运行
    server.run().unwrap_or_else(|e| {
        logger.error(&format!("Server error: {}", e));
    });

    // 服务器关闭时记录日志并打印统计信息
    logger.info("=== Server Shutdown ===");
    server.print_stats();
}

#[cfg(test)]
mod tests {
    use super::*;
    use compiler::{compile_and_run, CodeCompiler, Compiler};
    use config::{AppConfig, LogLevel};
    use errors::CodeError;
    use models::{CodeRequest, CodeResponse, CompileResult, ExecutionStats, Language};

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("python"), Some(Language::Python));
        assert_eq!(Language::from_str("java"), Some(Language::Java));
        assert_eq!(Language::from_str("c"), Some(Language::C));
        assert_eq!(Language::from_str("cpp"), Some(Language::Cpp));
        assert_eq!(Language::from_str("c++"), Some(Language::Cpp));
        assert_eq!(Language::from_str("rust"), Some(Language::Rust));
        assert_eq!(Language::from_str("unknown"), None);
    }

    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::Python.as_str(), "python");
        assert_eq!(Language::Java.as_str(), "java");
        assert_eq!(Language::C.as_str(), "c");
        assert_eq!(Language::Cpp.as_str(), "cpp");
        assert_eq!(Language::Rust.as_str(), "rust");
    }

    #[test]
    fn test_language_get_compiler() {
        assert_eq!(Language::Python.get_compiler(), "python");
        assert_eq!(Language::Java.get_compiler(), "javac");
        assert_eq!(Language::C.get_compiler(), "gcc");
        assert_eq!(Language::Cpp.get_compiler(), "g++");
        assert_eq!(Language::Rust.get_compiler(), "rustc");
    }

    #[test]
    fn test_language_get_display_name() {
        assert_eq!(Language::Python.get_display_name(), "Python");
        assert_eq!(Language::Java.get_display_name(), "Java");
        assert_eq!(Language::C.get_display_name(), "C");
        assert_eq!(Language::Cpp.get_display_name(), "C++");
        assert_eq!(Language::Rust.get_display_name(), "Rust");
    }

    #[test]
    fn test_compiler_trait() {
        let compiler = Compiler::new(Language::Python);
        let result = compiler.run("print('test')");
        assert!(result.is_ok());
        let compile_result = result.unwrap();
        assert!(compile_result.success);
        assert!(compile_result.stdout.contains("test"));
    }

    #[test]
    fn test_compile_and_run_python() {
        let result = compile_and_run(&Language::Python, "print('Hello, Python!')");
        assert!(result.success);
        assert!(result.output.contains("Hello, Python!"));
    }

    #[test]
    fn test_compile_and_run_python_error() {
        let result = compile_and_run(&Language::Python, "print(undefined_variable)");
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_compile_and_run_java() {
        let code = "public class Main { public static void main(String[] args) { System.out.println(\"Hello, Java!\"); } }";
        let result = compile_and_run(&Language::Java, code);
        assert!(result.success);
        assert!(result.output.contains("Hello, Java!"));
    }

    #[test]
    fn test_compile_and_run_c() {
        let code = "#include <stdio.h>\nint main() { printf(\"Hello, C!\\n\"); return 0; }";
        let result = compile_and_run(&Language::C, code);
        assert!(result.success);
        assert!(result.output.contains("Hello, C!"));
    }

    #[test]
    fn test_compile_and_run_cpp() {
        let code = "#include <iostream>\nint main() { std::cout << \"Hello, C++!\" << std::endl; return 0; }";
        let result = compile_and_run(&Language::Cpp, code);
        assert!(result.success);
        assert!(result.output.contains("Hello, C++!"));
    }

    #[test]
    fn test_compile_and_run_rust() {
        let code = "fn main() { println!(\"Hello, Rust!\"); }";
        let result = compile_and_run(&Language::Rust, code);
        assert!(result.success);
        assert!(result.output.contains("Hello, Rust!"));
    }

    #[test]
    fn test_code_response_success() {
        let response = CodeResponse::success("output".to_string(), Some(0));
        assert!(response.success);
        assert_eq!(response.output, "output");
        assert_eq!(response.exit_code, Some(0));
    }

    #[test]
    fn test_code_response_error() {
        let response = CodeResponse::error("error".to_string(), Some(1));
        assert!(!response.success);
        assert_eq!(response.errors, "error");
        assert_eq!(response.exit_code, Some(1));
    }

    #[test]
    fn test_code_response_with_execution_time() {
        let response = CodeResponse::success("output".to_string(), Some(0))
            .with_execution_time(123);
        assert_eq!(response.execution_time, Some(123));
    }

    #[test]
    fn test_code_error_language_not_supported() {
        let error = CodeError::LanguageNotSupported("test".to_string());
        assert_eq!(error.to_string(), "Language not supported: test");
    }

    #[test]
    fn test_code_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
        let code_error: CodeError = io_error.into();
        assert!(matches!(code_error, CodeError::FileError(_)));
    }

    #[test]
    fn test_code_response_to_json() {
        let response = CodeResponse::success("test output".to_string(), Some(0));
        let json = response.to_json();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"output\":\"test output\""));
        assert!(json.contains("\"exit_code\":0"));
    }

    #[test]
    fn test_code_request_new() {
        let request = CodeRequest::new("python", "print('test')");
        assert_eq!(request.language, "python");
        assert_eq!(request.code, "print('test')");
        assert_eq!(request.timeout, None);
    }

    #[test]
    fn test_code_request_with_timeout() {
        let request = CodeRequest::new("python", "print('test')").with_timeout(30);
        assert_eq!(request.timeout, Some(30));
    }

    #[test]
    fn test_code_request_parse_language() {
        let request = CodeRequest::new("python", "code");
        assert_eq!(request.parse_language(), Some(Language::Python));
    }

    #[test]
    fn test_execution_stats_new() {
        let stats = ExecutionStats::new(Language::Python);
        assert_eq!(stats.language, Language::Python);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
    }

    #[test]
    fn test_execution_stats_record_success() {
        let mut stats = ExecutionStats::new(Language::Python);
        stats.record_success(100);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.total_execution_time_ms, 100);
    }

    #[test]
    fn test_execution_stats_record_failure() {
        let mut stats = ExecutionStats::new(Language::Python);
        stats.record_failure();
        assert_eq!(stats.failure_count, 1);
    }

    #[test]
    fn test_execution_stats_get_total_count() {
        let mut stats = ExecutionStats::new(Language::Python);
        stats.record_success(100);
        stats.record_failure();
        assert_eq!(stats.get_total_count(), 2);
    }

    #[test]
    fn test_execution_stats_average_time() {
        let mut stats = ExecutionStats::new(Language::Python);
        stats.record_success(100);
        stats.record_success(200);
        assert_eq!(stats.get_average_time_ms(), 150.0);
    }

    #[test]
    fn test_server_config() {
        let config = ServerConfig::new("127.0.0.1", 3000)
            .with_max_workers(5)
            .with_timeout(10);
        assert_eq!(config.get_host(), "127.0.0.1");
        assert_eq!(config.get_port(), 3000);
        assert_eq!(config.get_max_workers(), 5);
    }

    #[test]
    fn test_compile_result_success() {
        let result = CompileResult::success("output".to_string(), Some(0));
        assert!(result.success);
        assert_eq!(result.stdout, "output");
    }

    #[test]
    fn test_compile_result_error() {
        let result = CompileResult::error("error".to_string(), Some(1));
        assert!(!result.success);
        assert_eq!(result.stderr, "error");
    }

    #[test]
    fn test_compiler_compile_python() {
        let compiler = Compiler::new(Language::Python);
        let result = compiler.compile("print('test')");
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::new();
        assert_eq!(config.get_server_config().get_host(), "127.0.0.1");
        assert_eq!(config.get_server_config().get_port(), 3000);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("trace"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("warn"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Trace.as_str(), "TRACE");
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_app_config_validate() {
        let config = AppConfig::new();
        assert!(config.validate().is_ok());
    }
}

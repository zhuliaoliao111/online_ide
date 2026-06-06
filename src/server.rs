//HTTP 服务器模块
//提供基于 TCP 的简易 HTTP 服务器实现，支持代码编译、静态分析等 API 端点。

use super::analyzer::analyze_code;
use super::compiler::compile_and_run;
use super::logging::Logger;
use super::models::{AnalysisResponse, CodeResponse, ExecutionStats, Language};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 服务器配置结构体
#[derive(Debug, Clone)]
pub struct ServerConfig {
    host: String,              // 绑定的主机地址
    port: u16,                 // 监听的端口号
    max_workers: usize,        // 最大工作线程数
    request_timeout_secs: u64, // 请求超时时间（秒）
}

impl ServerConfig {
    /// 创建新的服务器配置
    pub fn new(host: &str, port: u16) -> Self {
        ServerConfig {
            host: host.to_string(),
            port,
            max_workers: 10,          // 默认最大工作线程数
            request_timeout_secs: 30, // 默认超时时间
        }
    }

    /// 设置最大工作线程数（链式调用）
    pub fn with_max_workers(mut self, max_workers: usize) -> Self {
        self.max_workers = max_workers;
        self
    }

    /// 设置请求超时时间（链式调用）
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.request_timeout_secs = timeout_secs;
        self
    }

    /// 获取主机地址
    #[allow(dead_code)]
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// 获取端口号
    #[allow(dead_code)]
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// 获取最大工作线程数
    #[allow(dead_code)]
    pub fn get_max_workers(&self) -> usize {
        self.max_workers
    }
}

/// CodeServer 结构体 - 核心服务器实例
pub struct CodeServer {
    config: ServerConfig,                                 // 服务器配置
    request_count: Arc<Mutex<usize>>,                     // 请求计数器（线程安全）
    stats: Arc<Mutex<HashMap<Language, ExecutionStats>>>, // 各语言执行统计
    start_time: Instant,                                  // 服务器启动时间
    logger: Arc<Mutex<Logger>>,                           // 日志记录器
}

impl CodeServer {
    /// 创建新的 CodeServer 实例
    pub fn new(config: ServerConfig) -> Self {
        // 初始化各语言的执行统计
        let mut stats = HashMap::new();
        for lang in [
            Language::Python,
            Language::Java,
            Language::C,
            Language::Cpp,
            Language::Rust,
        ] {
            stats.insert(lang, ExecutionStats::new(lang));
        }

        // 创建默认日志记录器
        let logger =
            Logger::new(&super::config::LoggingConfig::default()).expect("Failed to create logger");

        CodeServer {
            config,
            request_count: Arc::new(Mutex::new(0)),
            stats: Arc::new(Mutex::new(stats)),
            start_time: Instant::now(),
            logger: Arc::new(Mutex::new(logger)),
        }
    }

    /// 配置自定义日志记录器（链式调用）
    pub fn with_logger(mut self, logging_config: &super::config::LoggingConfig) -> Self {
        if let Ok(logger) = Logger::new(logging_config) {
            self.logger = Arc::new(Mutex::new(logger));
        }
        self
    }

    /// 启动服务器并开始监听请求
    pub fn run(&self) -> std::io::Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr)?;

        // 记录服务器启动信息
        self.logger
            .lock()
            .unwrap()
            .log_server_start(&self.config.host, self.config.port);
        self.logger
            .lock()
            .unwrap()
            .info(&format!("Max workers: {}", self.config.max_workers));
        self.logger.lock().unwrap().info(&format!(
            "Request timeout: {} seconds",
            self.config.request_timeout_secs
        ));

        // 克隆共享状态供线程使用
        let request_count_clone = Arc::clone(&self.request_count);
        let stats_clone = Arc::clone(&self.stats);
        let logger_clone = Arc::clone(&self.logger);

        // 循环接受客户端连接
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let request_count = Arc::clone(&request_count_clone);
                    let stats = Arc::clone(&stats_clone);
                    let logger = Arc::clone(&logger_clone);
                    let timeout = self.config.request_timeout_secs;

                    // 为每个连接创建新线程处理
                    thread::spawn(move || {
                        *request_count.lock().unwrap() += 1;
                        if let Err(e) = handle_client(stream, stats, timeout, logger) {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    self.logger
                        .lock()
                        .unwrap()
                        .error(&format!("Failed to accept connection: {}", e));
                }
            }
        }

        Ok(())
    }

    /// 获取总请求数
    pub fn get_request_count(&self) -> usize {
        *self.request_count.lock().unwrap()
    }

    /// 获取服务器运行时间
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 获取各语言执行统计
    pub fn get_stats(&self) -> HashMap<Language, ExecutionStats> {
        self.stats.lock().unwrap().clone()
    }

    /// 打印服务器统计信息
    pub fn print_stats(&self) {
        let stats = self.get_stats();
        println!("\n=== Server Statistics ===");
        println!("Total requests: {}", self.get_request_count());
        println!("Uptime: {:?}", self.get_uptime());
        println!("Language statistics:");
        for (lang, stat) in stats {
            println!(
                "  {}: {} success, {} failure, avg time: {:.2}ms",
                lang.get_display_name(),
                stat.success_count,
                stat.failure_count,
                stat.get_average_time_ms()
            );
        }
    }
}

/// 处理单个客户端请求
fn handle_client(
    mut stream: TcpStream,
    stats: Arc<Mutex<HashMap<Language, ExecutionStats>>>,
    timeout: u64,
    logger: Arc<Mutex<Logger>>,
) -> std::io::Result<()> {
    let start_time = Instant::now();

    // 设置读写超时
    stream.set_read_timeout(Some(Duration::from_secs(timeout)))?;
    stream.set_write_timeout(Some(Duration::from_secs(timeout)))?;

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();

    // 读取请求行
    reader.read_line(&mut request_line)?;

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    // 读取请求体（仅 POST 请求）
    let mut body = String::new();
    if method == "POST" {
        let mut content_length = 0;

        // 解析请求头，获取 Content-Length
        for header in reader.by_ref().lines() {
            let header = header?;

            if header.starts_with("Content-Length:") {
                content_length = header
                    .split(":")
                    .nth(1)
                    .unwrap_or("0")
                    .trim()
                    .parse::<usize>()
                    .unwrap_or(0);
            }

            if header.is_empty() {
                break; // 空行表示头部结束
            }
        }

        // 读取请求体内容
        let mut buffer = vec![0; content_length];
        reader.read_exact(&mut buffer)?;
        body = String::from_utf8_lossy(&buffer).to_string();
    }

    // 根据请求方法和路径分发处理
    let response = match (method, path) {
        ("GET", "/") => handle_index(), // 返回首页
        ("POST", "/api/compile") => {
            // 编译执行代码
            let result = handle_compile(&body, &stats, &logger);
            let execution_time = start_time.elapsed().as_millis() as u64;
            format_response_with_time(&result, execution_time)
        }
        ("POST", "/api/analyze") => {
            // 静态代码分析
            let result = handle_analyze(&body, &logger);
            format_analysis_response(&result)
        }

        ("GET", "/health") => {
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\nOK".to_string()
        } // 健康检查
        ("GET", "/stats") => handle_stats(&stats), // 获取统计信息
        _ => "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\nNot Found"
            .to_string(), // 未找到
    };

    // 发送响应
    stream.write_all(response.as_bytes())?;
    Ok(())
}

/// 处理首页请求 - 返回前端 HTML
fn handle_index() -> String {
    match std::fs::read_to_string("frontend/index.html") {
        Ok(content) => format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
                content.len(),
                content
            ),
        Err(_) => "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\nFailed to read index.html".to_string(),
    }
}

/// 处理代码编译请求
fn handle_compile(
    body: &str,
    stats: &Arc<Mutex<HashMap<Language, ExecutionStats>>>,
    logger: &Arc<Mutex<Logger>>,
) -> CodeResponse {
    let start_time = Instant::now();
    let mut language_str = String::new();
    let mut code = String::new();
    let mut input = String::new();

    logger.lock().unwrap().debug(&format!(
        "Compile request received - Body length: {}",
        body.len()
    ));
    logger
        .lock()
        .unwrap()
        .debug(&format!("Request body: {}", body));

    // 解析请求体（支持 JSON 和表单格式）
    if body.starts_with("{") {
        // JSON 格式
        if let Ok(parsed) = parse_json_body(body) {
            language_str = parsed.0;
            code = parsed.1;
            input = parsed.2;
            logger.lock().unwrap().debug(&format!(
                "JSON parsed successfully - Language: {}, Code length: {}, Input length: {}",
                language_str,
                code.len(),
                input.len()
            ));
        } else {
            logger.lock().unwrap().error("Failed to parse JSON body");
            return CodeResponse::error("Failed to parse JSON request body".to_string(), None);
        }
    } else {
        // 表单格式（application/x-www-form-urlencoded）
        let parts: Vec<&str> = body.split("&").collect();
        for part in parts {
            if part.starts_with("language=") {
                language_str = url_decode(part.split("=").nth(1).unwrap_or(""));
            } else if part.starts_with("code=") {
                code = url_decode(part.split("=").nth(1).unwrap_or(""));
            } else if part.starts_with("input=") {
                input = url_decode(part.split("=").nth(1).unwrap_or(""));
            }
        }
        logger.lock().unwrap().debug(&format!(
            "Form parsed - Language: {}, Code length: {}, Input length: {}",
            language_str,
            code.len(),
            input.len()
        ));
    }

    // 解析语言类型
    let language = match Language::from_str(&language_str) {
        Some(lang) => lang,
        None => {
            let result =
                CodeResponse::error(format!("Language not supported: {}", language_str), None);
            logger
                .lock()
                .unwrap()
                .error(&format!("Language not supported: {}", language_str));
            update_stats(stats, &language_str, &result, 0);
            return result;
        }
    };

    // 执行编译运行
    let input_opt = if input.is_empty() {
        None
    } else {
        Some(input.as_str())
    };
    let result = compile_and_run(&language, &code, input_opt);
    let exec_time = start_time.elapsed().as_millis() as u64;

    // 记录日志和统计
    if result.success {
        logger.lock().unwrap().debug(&format!(
            "Execution successful - Output length: {}",
            result.output.len()
        ));
    } else {
        logger
            .lock()
            .unwrap()
            .error(&format!("Execution failed - Errors: {}", result.errors));
    }

    logger
        .lock()
        .unwrap()
        .log_request(&language_str, result.success, exec_time);
    update_stats(stats, &language_str, &result, exec_time);
    result
}

/// 解析 JSON 请求体（简易实现）
fn parse_json_body(body: &str) -> Result<(String, String, String), ()> {
    let body = body.trim();

    // 解析 language 字段
    let language_start = body.find("\"language\"").ok_or(())?;
    let language_colon = body[language_start..].find(':').ok_or(())? + language_start;
    let language_quote_start =
        body[language_colon + 1..].find('"').ok_or(())? + language_colon + 1 + 1;
    let language_quote_end =
        body[language_quote_start..].find('"').ok_or(())? + language_quote_start;
    let language_str = body[language_quote_start..language_quote_end].to_string();

    // 解析 code 字段
    let code_start = body.find("\"code\"").ok_or(())?;
    let code_colon = body[code_start..].find(':').ok_or(())? + code_start;
    let code_quote_start = body[code_colon + 1..].find('"').ok_or(())? + code_colon + 1 + 1;

    // 确定 code 字段的结束位置
    let input_start = body.find("\"input\"");
    let code_quote_end = if let Some(input_start) = input_start {
        let input_colon = body[input_start..].find(':').ok_or(())? + input_start;
        let before_input = &body[..input_colon];
        before_input.rfind('"').ok_or(())?
    } else {
        body.rfind('"').ok_or(())?
    };

    // 处理转义字符
    let code_str = body[code_quote_start..code_quote_end]
        .replace("\\n", "\n")
        .replace("\\\"", "\"");

    // 解析 input 字段（可选）
    let input_str = if let Some(input_start) = input_start {
        let input_colon = body[input_start..].find(':').ok_or(())? + input_start;
        let input_quote_start = body[input_colon + 1..].find('"').ok_or(())? + input_colon + 1 + 1;
        let input_quote_end = body[input_quote_start..].find('"').ok_or(())? + input_quote_start;
        body[input_quote_start..input_quote_end]
            .replace("\\n", "\n")
            .replace("\\\"", "\"")
    } else {
        String::new()
    };

    Ok((language_str, code_str, input_str))
}

/// 更新执行统计
fn update_stats(
    stats: &Arc<Mutex<HashMap<Language, ExecutionStats>>>,
    language_str: &str,
    result: &CodeResponse,
    exec_time: u64,
) {
    if let Some(language) = Language::from_str(language_str) {
        let mut stats_map = stats.lock().unwrap();
        if let Some(stat) = stats_map.get_mut(&language) {
            if result.success {
                stat.record_success(exec_time);
            } else {
                stat.record_failure();
            }
        }
    }
}

/// 处理代码分析请求
fn handle_analyze(body: &str, logger: &Arc<Mutex<Logger>>) -> AnalysisResponse {
    let mut language_str = String::new();
    let mut code = String::new();

    logger.lock().unwrap().debug(&format!(
        "Analyze request received - Body length: {}",
        body.len()
    ));

    // 解析请求体
    if body.starts_with("{") {
        if let Ok(parsed) = parse_json_body(body) {
            language_str = parsed.0;
            code = parsed.1;
            logger.lock().unwrap().debug(&format!(
                "JSON parsed - Language: {}, Code length: {}",
                language_str,
                code.len()
            ));
        } else {
            logger.lock().unwrap().error("Failed to parse JSON body");
            return AnalysisResponse {
                success: false,
                issues: vec![],
                execution_time_ms: 0,
                analyzer_version: "error".to_string(),
            };
        }
    } else {
        let parts: Vec<&str> = body.split("&").collect();
        for part in parts {
            if part.starts_with("language=") {
                language_str = url_decode(part.split("=").nth(1).unwrap_or(""));
            } else if part.starts_with("code=") {
                code = url_decode(part.split("=").nth(1).unwrap_or(""));
            }
        }
    }

    // 解析语言类型
    let language = match Language::from_str(&language_str) {
        Some(lang) => lang,
        None => {
            logger
                .lock()
                .unwrap()
                .error(&format!("Language not supported: {}", language_str));
            return AnalysisResponse {
                success: false,
                issues: vec![],
                execution_time_ms: 0,
                analyzer_version: format!("error:{}", language_str),
            };
        }
    };

    // 执行代码分析
    logger.lock().unwrap().debug(&format!(
        "Starting analysis for language: {}",
        language.as_str()
    ));
    let result = analyze_code(&language, &code);
    logger.lock().unwrap().debug(&format!(
        "Analysis complete - Found {} issues",
        result.issues.len()
    ));

    result
}

/// 格式化分析响应为 HTTP 响应
fn format_analysis_response(result: &AnalysisResponse) -> String {
    let json = result.to_json();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        json.len(),
        json
    )
}

/// 处理统计信息请求
fn handle_stats(stats: &Arc<Mutex<HashMap<Language, ExecutionStats>>>) -> String {
    let stats_map = stats.lock().unwrap();
    let mut json = String::from("{\"languages\":[");

    // 将统计数据转换为 JSON 数组
    let lang_stats: Vec<String> = stats_map
        .iter()
        .map(|(lang, stat)| {
            format!(
                "{{\"language\":\"{}\",\"success_count\":{},\"failure_count\":{},\"avg_time_ms\":{:.2}}}",
                lang.as_str(),
                stat.success_count,
                stat.failure_count,
                stat.get_average_time_ms()
            )
        })
        .collect();

    json.push_str(&lang_stats.join(","));
    json.push_str("]}");

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        json.len(),
        json
    )
}

/// URL 解码函数（处理 %XX 编码和 + 空格）
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // 处理 %XX 编码
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            // 处理 + 表示空格
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}

/// 格式化编译响应（带执行时间）
fn format_response_with_time(result: &CodeResponse, execution_time: u64) -> String {
    let result_with_time = result.clone().with_execution_time(execution_time);
    let json = result_with_time.to_json();

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        json.len(),
        json
    )
}

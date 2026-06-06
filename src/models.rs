//数据模型模块

//定义项目中使用的核心数据结构和枚举类型。

use serde::Serialize;

/// 支持的编程语言枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Python,
    Java,
    C,
    Cpp,
    Rust,
}

impl Language {
    /// 从字符串解析语言类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "python" => Some(Language::Python),
            "java" => Some(Language::Java),
            "c" => Some(Language::C),
            "cpp" | "c++" => Some(Language::Cpp), // 支持两种写法
            "rust" => Some(Language::Rust),
            _ => None,
        }
    }

    /// 获取语言的字符串表示
    pub fn as_str(self) -> &'static str {
        match self {
            Language::Python => "python",
            Language::Java => "java",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Rust => "rust",
        }
    }

    /// 获取该语言对应的编译器命令
    pub fn get_compiler(&self) -> &'static str {
        match self {
            Language::Python => "python",
            Language::Java => "javac",
            Language::C => "gcc",
            Language::Cpp => "g++",
            Language::Rust => "rustc",
        }
    }

    /// 获取该语言源文件的扩展名
    pub fn get_source_extension(&self) -> &'static str {
        match self {
            Language::Python => ".py",
            Language::Java => ".java",
            Language::C => ".c",
            Language::Cpp => ".cpp",
            Language::Rust => ".rs",
        }
    }

    /// 获取语言的显示名称（用于 UI）
    pub fn get_display_name(&self) -> &'static str {
        match self {
            Language::Python => "Python",
            Language::Java => "Java",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Rust => "Rust",
        }
    }
}

/// 代码请求结构体
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CodeRequest {
    pub language: String,     // 语言类型
    pub code: String,         // 代码内容
    pub timeout: Option<u32>, // 超时时间（可选）
}

#[allow(dead_code)]
impl CodeRequest {
    /// 创建新的代码请求
    pub fn new(language: &str, code: &str) -> Self {
        CodeRequest {
            language: language.to_string(),
            code: code.to_string(),
            timeout: None,
        }
    }

    /// 设置超时时间（链式调用）
    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 解析语言类型
    pub fn parse_language(&self) -> Option<Language> {
        Language::from_str(&self.language)
    }
}

/// 代码执行响应结构体
#[derive(Debug, Clone, Serialize)]
pub struct CodeResponse {
    pub success: bool,               // 是否执行成功
    pub output: String,              // 标准输出
    pub errors: String,              // 错误信息
    pub exit_code: Option<i32>,      // 退出码
    pub execution_time: Option<u64>, // 执行时间（毫秒）
}

impl CodeResponse {
    /// 创建成功响应
    pub fn success(output: String, exit_code: Option<i32>) -> Self {
        CodeResponse {
            success: true,
            output,
            errors: String::new(),
            exit_code,
            execution_time: None,
        }
    }

    /// 创建错误响应
    pub fn error(errors: String, exit_code: Option<i32>) -> Self {
        CodeResponse {
            success: false,
            output: String::new(),
            errors,
            exit_code,
            execution_time: None,
        }
    }

    /// 设置执行时间（链式调用）
    pub fn with_execution_time(mut self, time_ms: u64) -> Self {
        self.execution_time = Some(time_ms);
        self
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{\"success\":false,\"output\":\"\",\"errors\":\"JSON serialization failed\",\"exit_code\":null,\"execution_time\":null}".to_string())
    }
}

/// 编译结果结构体
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub success: bool,          // 是否成功
    pub stdout: String,         // 标准输出
    pub stderr: String,         // 标准错误
    pub exit_code: Option<i32>, // 退出码
}

impl CompileResult {
    /// 创建成功的编译结果
    pub fn success(stdout: String, exit_code: Option<i32>) -> Self {
        CompileResult {
            success: true,
            stdout,
            stderr: String::new(),
            exit_code,
        }
    }

    /// 创建失败的编译结果
    pub fn error(stderr: String, exit_code: Option<i32>) -> Self {
        CompileResult {
            success: false,
            stdout: String::new(),
            stderr,
            exit_code,
        }
    }
}

/// 执行统计结构体
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExecutionStats {
    pub language: Language,           // 语言类型
    pub success_count: u64,           // 成功次数
    pub failure_count: u64,           // 失败次数
    pub total_execution_time_ms: u64, // 总执行时间（毫秒）
}

impl ExecutionStats {
    /// 创建新的统计实例
    pub fn new(language: Language) -> Self {
        ExecutionStats {
            language,
            success_count: 0,
            failure_count: 0,
            total_execution_time_ms: 0,
        }
    }

    /// 记录一次成功执行
    pub fn record_success(&mut self, time_ms: u64) {
        self.success_count += 1;
        self.total_execution_time_ms += time_ms;
    }

    /// 记录一次失败执行
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
    }

    /// 获取总执行次数
    #[allow(dead_code)]
    pub fn get_total_count(&self) -> u64 {
        self.success_count + self.failure_count
    }

    /// 获取平均执行时间（毫秒）
    pub fn get_average_time_ms(&self) -> f64 {
        if self.success_count == 0 {
            0.0
        } else {
            self.total_execution_time_ms as f64 / self.success_count as f64
        }
    }
}

/// 代码分析问题结构体
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisIssue {
    pub severity: String,      // 严重程度：error, warning, info
    pub message: String,       // 问题描述
    pub line: usize,           // 所在行
    pub column: Option<usize>, // 所在列（可选）
    pub rule: String,          // 规则名称
}

impl AnalysisIssue {
    /// 创建新的分析问题
    #[allow(dead_code)]
    pub fn new(severity: &str, message: &str, line: usize, rule: &str) -> Self {
        AnalysisIssue {
            severity: severity.to_string(),
            message: message.to_string(),
            line,
            column: None,
            rule: rule.to_string(),
        }
    }
}

/// 代码分析响应结构体
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisResponse {
    pub success: bool,              // 是否分析成功
    pub issues: Vec<AnalysisIssue>, // 发现的问题列表
    pub execution_time_ms: u64,     // 分析耗时（毫秒）
    pub analyzer_version: String,   // 分析器版本信息
}

impl AnalysisResponse {
    /// 创建成功的分析响应
    #[allow(dead_code)]
    pub fn success(issues: Vec<AnalysisIssue>, exec_time: u64, version: &str) -> Self {
        AnalysisResponse {
            success: true,
            issues,
            execution_time_ms: exec_time,
            analyzer_version: version.to_string(),
        }
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{\"success\":false,\"issues\":[],\"execution_time_ms\":0,\"analyzer_version\":\"\"}".to_string())
    }
}

//! # 代码分析模块
//! 
//! 提供多语言代码静态分析功能，支持 Python、Java、C、C++、Rust 五种语言的代码检查。
//! 使用各语言的原生分析工具（如 pylint、javac、gcc、clippy 等）进行静态分析。

use super::models::{AnalysisIssue, AnalysisResponse, Language};
use std::process::{Command, Stdio};
use std::time::Instant;

/// 代码分析器结构体
pub struct CodeAnalyzer;

impl CodeAnalyzer {
    /// 创建代码分析器实例
    pub fn new() -> Self {
        CodeAnalyzer
    }

    /// 分析指定语言的代码
    /// 
    /// # 参数
    /// - `language`: 编程语言类型
    /// - `code`: 待分析的代码内容
    /// 
    /// # 返回值
    /// 分析响应，包含发现的问题列表和执行时间
    pub fn analyze(&self, language: &Language, code: &str) -> AnalysisResponse {
        let start_time = Instant::now();
        
        let issues = match language {
            Language::Python => self.analyze_python(code),
            Language::Java => self.analyze_java(code),
            Language::C => self.analyze_c(code),
            Language::Cpp => self.analyze_cpp(code),
            Language::Rust => self.analyze_rust(code),
        };

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        AnalysisResponse {
            success: true,
            issues,
            execution_time_ms: elapsed_ms,
            analyzer_version: self.get_analyzer_version(language),
        }
    }

    fn analyze_python(&self, code: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        let temp_dir = tempfile::tempdir().ok();
        let temp_dir = match temp_dir {
            Some(d) => d,
            None => return issues,
        };

        let source_path = temp_dir.path().join("analysis_target.py");
        if let Err(_) = std::fs::write(&source_path, code) {
            return issues;
        }

        // 优先使用 pylint 进行分析
        let output = Command::new("pylint")
            .arg("--output-format=text")
            .arg("--exit-zero")
            .arg(&source_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            issues.extend(self.parse_pylint_output(&stdout));
            if stderr.len() > 0 && issues.is_empty() {
                issues.extend(self.parse_pylint_output(&stderr));
            }
        }

        // 如果 pylint 不可用，尝试 pyflakes 或基础语法检查
        if issues.is_empty() {
            let output = Command::new("python")
                .arg("-m")
                .arg("py_compile")
                .arg(&source_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();

            if let Ok(output) = output {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    issues.push(AnalysisIssue {
                        severity: "error".to_string(),
                        message: stderr.trim().to_string(),
                        line: self.extract_line_number(&stderr, 1),
                        column: Some(0),
                        rule: "syntax".to_string(),
                    });
                }
            }
        }

        issues
    }

    fn parse_pylint_output(&self, output: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        for line in output.lines() {
            if line.contains(":") && !line.starts_with("****") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let filename = parts.get(0).unwrap_or(&"");
                    let line_num = parts.get(1).unwrap_or(&"0").parse::<usize>().unwrap_or(0);
                    let column_str = parts.get(2).unwrap_or(&"0");
                    let column = column_str.parse::<usize>().unwrap_or(0);
                    let rest = parts[3..].join(":");
                    
                    let severity = if rest.contains("(error)") || rest.contains("(F)")
                        || rest.contains("(E)") {
                        "error"
                    } else if rest.contains("(warning)") || rest.contains("(W)") {
                        "warning"
                    } else {
                        "info"
                    };

                    let rule = if rest.contains("(C)") {
                        "convention"
                    } else if rest.contains("(R)") {
                        "refactor"
                    } else if rest.contains("(W)") {
                        "warning"
                    } else if rest.contains("(E)") {
                        "error"
                    } else if rest.contains("(F)") {
                        "fatal"
                    } else {
                        "style"
                    };

                    issues.push(AnalysisIssue {
                        severity: severity.to_string(),
                        message: rest.trim().to_string(),
                        line: line_num,
                        column: Some(column),
                        rule: rule.to_string(),
                    });
                }
            }
        }
        
        issues
    }

    fn analyze_java(&self, code: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        let temp_dir = tempfile::tempdir().ok();
        let temp_dir = match temp_dir {
            Some(d) => d,
            None => return issues,
        };

        let source_path = temp_dir.path().join("Main.java");
        if let Err(_) = std::fs::write(&source_path, code) {
            return issues;
        }

        // 使用 javac 和 -Xlint 标志进行静态分析
        let output = Command::new("javac")
            .arg("-Xlint:all")
            .arg("-Xlint:-options")
            .arg(&source_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        if let Ok(output) = output {
            let stderr = String::from_utf8_lossy(&output.stderr);
            issues.extend(self.parse_javac_output(&stderr));
        }

        issues
    }

    fn parse_javac_output(&self, output: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        for line in output.lines() {
            if line.contains("warning:") || line.contains("error:") {
                let line_num = self.extract_line_number(line, 1);
                let message = line.to_string();
                
                let severity = if line.contains("error:") {
                    "error"
                } else {
                    "warning"
                };

                let rule = if line.contains("unchecked") {
                    "unchecked"
                } else if line.contains("deprecation") {
                    "deprecation"
                } else if line.contains("serial") {
                    "serial"
                } else if line.contains("rawtypes") {
                    "rawtypes"
                } else {
                    "lint"
                };

                issues.push(AnalysisIssue {
                    severity: severity.to_string(),
                    message,
                    line: line_num,
                    column: None,
                    rule: rule.to_string(),
                });
            }
        }
        
        issues
    }

    fn analyze_c(&self, code: &str) -> Vec<AnalysisIssue> {
        self.analyze_c_cpp(code, "gcc", ".c")
    }

    fn analyze_cpp(&self, code: &str) -> Vec<AnalysisIssue> {
        self.analyze_c_cpp(code, "g++", ".cpp")
    }

    fn analyze_c_cpp(&self, code: &str, compiler: &str, extension: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        let temp_dir = tempfile::tempdir().ok();
        let temp_dir = match temp_dir {
            Some(d) => d,
            None => return issues,
        };

        let source_name = format!("main{}", extension);
        let source_path = temp_dir.path().join(&source_name);
        if let Err(_) = std::fs::write(&source_path, code) {
            return issues;
        }

        // 使用编译器的 -fsyntax-only 和 -Wall -Wextra 选项进行分析
        let output = Command::new(compiler)
            .args(&["-fsyntax-only", "-Wall", "-Wextra", "-pedantic", "-std=c11"])
            .arg(&source_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        if let Ok(output) = output {
            let stderr = String::from_utf8_lossy(&output.stderr);
            issues.extend(self.parse_gcc_output(&stderr, compiler));
        }

        // 如果 cppcheck 可用则尝试使用
        let cppcheck_output = Command::new("cppcheck")
            .arg("--quiet")
            .arg("--enable=all")
            .arg(&source_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        if let Ok(cppcheck_output) = cppcheck_output {
            let stdout = String::from_utf8_lossy(&cppcheck_output.stdout);
            let stderr = String::from_utf8_lossy(&cppcheck_output.stderr);
            issues.extend(self.parse_cppcheck_output(&stdout));
            issues.extend(self.parse_cppcheck_output(&stderr));
        }

        issues
    }

    fn parse_gcc_output(&self, output: &str, compiler: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        for line in output.lines() {
            if line.contains("warning:") || line.contains("error:") {
                let line_num = self.extract_line_number(line, 1);
                
                let severity = if line.contains("error:") {
                    "error"
                } else {
                    "warning"
                };

                let rule = if line.contains("unused") {
                    "unused-variable"
                } else if line.contains("implicit") {
                    "implicit-int"
                } else if line.contains("return") && line.contains("warning") {
                    "return-type"
                } else {
                    "lint"
                };

                issues.push(AnalysisIssue {
                    severity: severity.to_string(),
                    message: line.to_string(),
                    line: line_num,
                    column: None,
                    rule: rule.to_string(),
                });
            }
        }
        
        issues
    }

    fn parse_cppcheck_output(&self, output: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        for line in output.lines() {
            if line.contains("[") && line.contains("]") {
                let line_num = self.extract_line_number(line, 1);
                
                // cppcheck 格式：file:line:message: [rulename]
                let message_parts: Vec<&str> = line.split('[').collect();
                let message = message_parts.get(0).unwrap_or(&"").trim().to_string();
                let rule = if message_parts.len() > 1 {
                    message_parts[1].replace("]", "").to_string()
                } else {
                    "cppcheck".to_string()
                };

                let severity = if line.contains("(error)") {
                    "error"
                } else if line.contains("(warning)") || line.contains("(style)") {
                    "warning"
                } else {
                    "info"
                };

                issues.push(AnalysisIssue {
                    severity: severity.to_string(),
                    message,
                    line: line_num,
                    column: None,
                    rule,
                });
            }
        }
        
        issues
    }

    fn analyze_rust(&self, code: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        let temp_dir = tempfile::tempdir().ok();
        let temp_dir = match temp_dir {
            Some(d) => d,
            None => return issues,
        };

        let source_path = temp_dir.path().join("main.rs");
        if let Err(_) = std::fs::write(&source_path, code) {
            return issues;
        }

        // 使用 rustc --emit=metadata 进行语法和类型检查
        let output = Command::new("rustc")
            .args(&["--emit=metadata", "-Z", "unpretty=every-file_loops_normalized"])
            .arg(&source_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        if let Ok(output) = output {
            let stderr = String::from_utf8_lossy(&output.stderr);
            issues.extend(self.parse_rustc_output(&stderr));
        }

        // 尝试使用 clippy 进行更详细的分析
        let clippy_output = Command::new("cargo")
            .args(&["clippy", "--message-format=json"])
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        if let Ok(clippy_output) = clippy_output {
            let stdout = String::from_utf8_lossy(&clippy_output.stdout);
            let stderr = String::from_utf8_lossy(&clippy_output.stderr);
            issues.extend(self.parse_clippy_output(&stdout));
            issues.extend(self.parse_clippy_output(&stderr));
        }

        issues
    }

    fn parse_rustc_output(&self, output: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        for line in output.lines() {
            if line.contains("error") || line.contains("warning") {
                let line_num = self.extract_line_number(line, 1);
                
                let severity = if line.contains("error") {
                    "error"
                } else {
                    "warning"
                };

                let rule = if line.contains("unused") {
                    "unused"
                } else if line.contains("dead_code") {
                    "dead-code"
                } else if line.contains("unknown") {
                    "unknown-lint"
                } else {
                    "rustc"
                };

                issues.push(AnalysisIssue {
                    severity: severity.to_string(),
                    message: line.to_string(),
                    line: line_num,
                    column: None,
                    rule: rule.to_string(),
                });
            }
        }
        
        issues
    }

    fn parse_clippy_output(&self, output: &str) -> Vec<AnalysisIssue> {
        let mut issues = Vec::new();
        
        // 对 clippy 输出进行简单的文本解析
        for line in output.lines() {
            // 尝试从以下格式提取行号：
            // src/main.rs:10:5: warning: ...
            //    |
            // 10 | let x = 1;
            //    | ^^^^^^ help: ...
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(line_num) = parts.get(1).unwrap_or(&"0").parse::<usize>() {
                    let message = parts.get(2..).unwrap_or(&[]).join(":");
                    if message.contains("warning") || message.contains("error") {
                        let severity = if message.contains("error") { "error" } else { "warning" };
                        issues.push(AnalysisIssue {
                            severity: severity.to_string(),
                            message: message.trim().to_string(),
                            line: line_num,
                            column: None,
                            rule: "clippy".to_string(),
                        });
                    }
                }
            }
        }
        
        issues
    }

    fn get_analyzer_version(&self, language: &Language) -> String {
        let tool = match language {
            Language::Python => "pylint/pyflakes",
            Language::Java => "javac",
            Language::C => "gcc",
            Language::Cpp => "g++",
            Language::Rust => "rustc/clippy",
        };
        format!("static-analysis/{}", tool)
    }

    fn extract_line_number(&self, text: &str, default: usize) -> usize {
        // 尝试从各种格式中提取行号，如 "file:line:col: message"
        let parts: Vec<&str> = text.split(':').collect();
        for part in &parts {
            if let Ok(num) = part.trim().parse::<usize>() {
                return num;
            }
        }
        default
    }
}

/// 代码分析的统一入口函数
/// 
/// # 参数
/// - `language`: 编程语言类型
/// - `code`: 待分析的代码内容
/// 
/// # 返回值
/// 分析响应，包含发现的问题列表和执行时间
pub fn analyze_code(language: &Language, code: &str) -> AnalysisResponse {
    let analyzer = CodeAnalyzer::new();
    analyzer.analyze(language, code)
}

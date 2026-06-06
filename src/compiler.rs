//! # 代码编译器模块
//! 
//! 提供跨语言代码编译和执行功能，支持 Python、Java、C、C++、Rust 五种语言。

use super::errors::{CodeError, Result};
use super::models::{CodeResponse, CompileResult, Language};
use std::io::Write;
use std::process::{Command, Stdio};

/// 代码编译器 trait - 定义编译执行接口
pub trait CodeCompiler {
    #[allow(dead_code)]
    fn compile(&self, code: &str) -> Result<CompileResult>;  // 仅编译（不执行）
    fn run(&self, code: &str) -> Result<CompileResult>;       // 编译并执行
    fn run_with_input(&self, code: &str, input: Option<&str>) -> Result<CompileResult>; // 带输入的执行
}

/// 编译器结构体
pub struct Compiler {
    language: Language,  // 目标语言
}

impl Compiler {
    /// 创建指定语言的编译器实例
    pub fn new(language: Language) -> Self {
        Compiler { language }
    }
}

impl CodeCompiler for Compiler {
    /// 编译并执行代码（带标准输入）
    fn run_with_input(&self, code: &str, input: Option<&str>) -> Result<CompileResult> {
        execute_code_with_input(&self.language, code, input)
    }
    
    /// 仅编译代码（不执行）
    fn compile(&self, code: &str) -> Result<CompileResult> {
        let temp_dir = tempfile::tempdir()?;
        let source_ext = self.language.get_source_extension();
        let source_path = temp_dir.path().join(format!("main{}", source_ext));

        // 将代码写入临时源文件
        std::fs::write(&source_path, code)?;

        let compiler = self.language.get_compiler();
        let output_path = temp_dir.path().join("main.exe");

        // 根据语言构建编译参数
        let args = match self.language {
            Language::Python => vec!["-c", code],  // Python 直接执行
            Language::Java => vec![source_path.to_str().unwrap()],
            Language::C => vec![
                source_path.to_str().unwrap(),
                "-o",
                output_path.to_str().unwrap(),
            ],
            Language::Cpp => vec![
                source_path.to_str().unwrap(),
                "-o",
                output_path.to_str().unwrap(),
            ],
            Language::Rust => vec![
                source_path.to_str().unwrap(),
                "-o",
                output_path.to_str().unwrap(),
            ],
        };

        // 执行编译命令
        let output = Command::new(compiler)
            .args(&args)
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();

        if output.status.success() {
            Ok(CompileResult::success(stdout, exit_code))
        } else {
            Err(CodeError::CompilationError(stderr))
        }
    }

    /// 编译并执行代码（无输入）
    fn run(&self, code: &str) -> Result<CompileResult> {
        execute_code(&self.language, code)
    }
}

/// 执行代码（无输入）
fn execute_code(language: &Language, code: &str) -> Result<CompileResult> {
    if matches!(language, Language::Python) {
        // Python 直接通过解释器执行
        let output = Command::new("python")
            .arg("-c")
            .arg(code)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();

        if output.status.success() {
            Ok(CompileResult::success(stdout, exit_code))
        } else {
            Ok(CompileResult::error(stderr, exit_code))
        }
    } else {
        // 编译型语言：先编译再执行
        let temp_dir = tempfile::tempdir()?;

        let source_ext = language.get_source_extension();
        let class_name = if matches!(language, Language::Java) {
            "Main"  // Java 必须使用 Main 类名
        } else {
            "main"
        };

        let source_path = temp_dir
            .path()
            .join(format!("{}{}", class_name, source_ext));
        let exe_path = temp_dir.path().join("main.exe");

        // 写入源文件
        std::fs::write(&source_path, code)?;

        let compiler = language.get_compiler();
        let compile_args = match language {
            Language::Java => vec![source_path.to_str().unwrap()],
            Language::C | Language::Cpp | Language::Rust => vec![
                source_path.to_str().unwrap(),
                "-o",
                exe_path.to_str().unwrap(),
            ],
            _ => unreachable!(),
        };

        // 执行编译
        let compile_output = Command::new(compiler)
            .args(&compile_args)
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        // 编译失败
        if !compile_output.status.success() {
            let stderr = String::from_utf8_lossy(&compile_output.stderr).to_string();
            return Ok(CompileResult::error(stderr, compile_output.status.code()));
        }

        // 编译成功，执行程序
        let runner = if matches!(language, Language::Java) {
            "java"  // Java 使用 java 命令运行
        } else {
            exe_path.to_str().unwrap()
        };

        let run_args = if matches!(language, Language::Java) {
            vec!["Main"]
        } else {
            vec![]
        };

        let run_output = Command::new(runner)
            .args(&run_args)
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();
        let exit_code = run_output.status.code();

        if run_output.status.success() {
            Ok(CompileResult::success(stdout, exit_code))
        } else {
            Ok(CompileResult::error(stderr, exit_code))
        }
    }
}

/// 执行代码（带标准输入）
fn execute_code_with_input(language: &Language, code: &str, input: Option<&str>) -> Result<CompileResult> {
    if matches!(language, Language::Python) {
        // Python 带输入执行
        let mut child = Command::new("python")
            .arg("-c")
            .arg(code)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // 写入标准输入
        if let Some(input_str) = input {
            if let Some(mut stdin) = child.stdin.as_mut() {
                stdin.write_all(input_str.as_bytes())?;
            }
        }

        let output = child.wait_with_output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();

        if output.status.success() {
            Ok(CompileResult::success(stdout, exit_code))
        } else {
            Ok(CompileResult::error(stderr, exit_code))
        }
    } else {
        // 编译型语言带输入执行
        let temp_dir = tempfile::tempdir()?;

        let source_ext = language.get_source_extension();
        let class_name = if matches!(language, Language::Java) {
            "Main"
        } else {
            "main"
        };

        let source_path = temp_dir
            .path()
            .join(format!("{}{}", class_name, source_ext));
        let exe_path = temp_dir.path().join("main.exe");

        std::fs::write(&source_path, code)?;

        let compiler = language.get_compiler();
        let compile_args = match language {
            Language::Java => vec![source_path.to_str().unwrap()],
            Language::C | Language::Cpp | Language::Rust => vec![
                source_path.to_str().unwrap(),
                "-o",
                exe_path.to_str().unwrap(),
            ],
            _ => unreachable!(),
        };

        // 编译
        let compile_output = Command::new(compiler)
            .args(&compile_args)
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !compile_output.status.success() {
            let stderr = String::from_utf8_lossy(&compile_output.stderr).to_string();
            return Ok(CompileResult::error(stderr, compile_output.status.code()));
        }

        // 执行（带输入）
        let runner = if matches!(language, Language::Java) {
            "java"
        } else {
            exe_path.to_str().unwrap()
        };

        let run_args = if matches!(language, Language::Java) {
            vec!["Main"]
        } else {
            vec![]
        };

        let mut child = Command::new(runner)
            .args(&run_args)
            .current_dir(temp_dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // 写入标准输入
        if let Some(input_str) = input {
            if let Some(mut stdin) = child.stdin.as_mut() {
                stdin.write_all(input_str.as_bytes())?;
            }
        }

        let run_output = child.wait_with_output()?;

        let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();
        let exit_code = run_output.status.code();

        if run_output.status.success() {
            Ok(CompileResult::success(stdout, exit_code))
        } else {
            Ok(CompileResult::error(stderr, exit_code))
        }
    }
}

/// 统一的编译执行入口函数
pub fn compile_and_run(language: &Language, code: &str, input: Option<&str>) -> CodeResponse {
    let compiler = Compiler::new(*language);

    match compiler.run_with_input(code, input) {
        Ok(result) => {
            if result.success {
                CodeResponse::success(result.stdout, result.exit_code)
            } else {
                let detailed_error = format!("Compilation or runtime error:\n{}", result.stderr);
                CodeResponse::error(detailed_error, result.exit_code)
            }
        }
        Err(e) => {
            let detailed_error = format!("Internal server error: {}\nLanguage: {}\nCode length: {}", 
                                         e.to_string(), language.as_str(), code.len());
            CodeResponse::error(detailed_error, None)
        }
    }
}

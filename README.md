# Online Code IDE Backend

一个基于 Rust 开发的在线代码编辑器后端服务，支持 Java/Python/C/C++/Rust 多种编程语言的编译、执行和静态分析。

## 功能特性

- 支持多种编程语言的编译和执行：
  - Python
  - Java
  - C
  - C++
  - Rust
- 代码静态分析功能，支持各语言的原生分析工具：
  - Python: pylint（不可用时回退到 `python -m py_compile` 进行语法检查）
  - Java: javac（带 `-Xlint:all -Xlint:-options`）
  - C: gcc（带 `-fsyntax-only -Wall -Wextra -pedantic -std=c11`），可用时附加 cppcheck
  - C++: g++（带 `-fsyntax-only -Wall -Wextra -pedantic -std=c++17`），可用时附加 cppcheck
  - Rust: rustc（`--emit=metadata --edition=2021`），可用时附加 cargo clippy
- 多线程并发请求处理
- 支持标准输入的代码执行
- 代码执行统计和监控
- 配置文件支持（自实现的 `key = value` 格式解析器，文件以 `.toml` 命名但非标准 TOML 解析）
- 结构化日志记录（支持多级别日志）
- 内置 Web 前端界面（集成 Monaco Editor）
- AI 代码助手集成（前端直连第三方大模型 API）
- 实时代码分析和诊断
- 跨域支持（CORS：API 响应设置 `Access-Control-Allow-Origin: *`）

## 技术栈

- Rust 2021 Edition
- 标准 TCP/HTTP 服务器实现（不依赖第三方 Web 框架）
- 多线程并发处理（基于 `std::thread`）
- 临时文件系统操作（`tempfile` 库）
- 时间处理和格式化（`time` 库）
- 序列化与 JSON 处理（`serde` + `serde_json`）

## 项目结构

```
online_ide/
├── Cargo.toml              # 项目配置和依赖
├── Cargo.lock              # 依赖锁定文件
├── README.md               # 项目说明文档
├── config.toml             # 配置文件（可选）
├── .gitignore              # Git 忽略文件配置
├── frontend/               # 前端资源
│   └── index.html          # 主界面（集成 Monaco Editor 和 AI 助手）
├── logs/                   # 日志目录
│   └── server.log          # 服务器日志文件
└── src/
    ├── main.rs             # 主入口文件
    ├── analyzer.rs         # 代码分析模块
    ├── compiler.rs         # 编译器模块
    ├── config.rs           # 配置管理模块
    ├── errors.rs           # 错误处理模块
    ├── logging.rs          # 日志系统模块
    ├── models.rs           # 数据模型模块
    └── server.rs           # HTTP 服务器模块
```

## 安装与运行

### 环境要求

- Rust 1.75+
- Python 3.x (用于 Python 代码执行)
- JDK (用于 Java 代码编译)
- GCC/G++ (用于 C/C++ 代码编译)
- Rustc (用于 Rust 代码编译)

### 构建项目

```bash
cargo build --release
```

### 运行项目

```bash
cargo run --release
```

或者运行编译后的二进制文件：

```bash
./target/release/code_ide_backend
```

服务器默认运行在 `http://127.0.0.1:3000`

### 测试

```bash
cargo test
```

### 代码格式化

```bash
cargo fmt
```

### 代码检查

```bash
cargo clippy
```

## API 接口

### 编译执行代码

**POST** `/api/compile`

支持 JSON 和表单两种请求格式。

JSON 格式请求体：
```json
{
  "language": "python",
  "code": "print('Hello, World!')",
  "input": "test input"
}
```

表单格式请求体：
```
language=python&code=print('Hello')&input=test
```

响应体：
```json
{
  "success": true,
  "output": "Hello, World!\n",
  "errors": "",
  "exit_code": 0,
  "execution_time": 123
}
```

### 代码静态分析

**POST** `/api/analyze`

请求体：
```json
{
  "language": "python",
  "code": "print('Hello')"
}
```

响应体：
```json
{
  "success": true,
  "issues": [
    {
      "severity": "warning",
      "message": "Unused variable 'x'",
      "line": 10,
      "column": 5,
      "rule": "unused-variable"
    }
  ],
  "execution_time_ms": 45,
  "analyzer_version": "static-analysis/pylint/pyflakes"
}
```

> 上述 `analyzer_version` 字段是 `CodeAnalyzer::get_analyzer_version` 返回的固定字符串前缀，各语言对应的实际取值如下：
> - Python → `static-analysis/pylint/pyflakes`
> - Java → `static-analysis/javac`
> - C → `static-analysis/gcc`
> - C++ → `static-analysis/g++`
> - Rust → `static-analysis/rustc/clippy`
>
> 注：Python 当前实际只调用 `pylint`（必要时回退到 `python -m py_compile`），并未真正调用 `pyflakes`，该字符串为对外展示的版本标签。

### 获取统计信息

**GET** `/stats`

响应体：
```json
{
  "languages": [
    {
      "language": "python",
      "success_count": 10,
      "failure_count": 2,
      "avg_time_ms": 123.4
    }
  ]
}
```

### 健康检查

**GET** `/health`

响应：`OK`

### 访问首页

**GET** `/`

返回前端 HTML 界面（集成 Monaco Editor 和 AI 助手）

## 配置文件

创建 `config.toml` 文件可以自定义服务器配置。**注：项目使用自定义的 `key = value` 解析器（见 `config.rs::parse_config_file`），不是标准 TOML 库**，因此不支持 TOML 的表头/嵌套数组/多行字符串等高级特性，仅支持点号分隔的扁平常量配置。

```toml
# Server Configuration
server.host = 127.0.0.1
server.port = 3000
server.max_workers = 10
server.timeout_secs = 30
server.max_request_size = 1048576

# Compiler Configuration
compiler.python_path = python
compiler.java_path = javac
compiler.gcc_path = gcc
compiler.gpp_path = g++
compiler.rustc_path = rustc
compiler.max_compile_time_secs = 60
compiler.max_run_time_secs = 30
compiler.temp_dir = 

# Logging Configuration
logging.level = info
logging.file_path = logs/server.log
logging.max_file_size_mb = 10
logging.max_backups = 5
```

### 配置说明

#### 服务器配置
- `server.host`: 服务器绑定地址（默认：127.0.0.1）
- `server.port`: 监听端口（默认：3000）
- `server.max_workers`: 最大工作线程数（默认：10）
- `server.timeout_secs`: 请求超时时间（默认：30秒）
- `server.max_request_size`: 最大请求大小（字节，默认：1MB）。**注意：当前实现仅解析该字段，未在 HTTP 请求处理中实际校验。**

#### 编译器配置
- `compiler.python_path`: Python 解释器路径（默认：`python`）
- `compiler.java_path`: Java 编译器路径（默认：`javac`）
- `compiler.gcc_path`: GCC 编译器路径（默认：`gcc`）
- `compiler.gpp_path`: G++ 编译器路径（默认：`g++`）
- `compiler.rustc_path`: Rust 编译器路径（默认：`rustc`）
- `compiler.max_compile_time_secs`: 最大编译时间（默认：60秒）。**注意：当前实现未对编译过程强制超时。**
- `compiler.max_run_time_secs`: 最大运行时间（默认：30秒）。**注意：当前实现未对运行过程强制超时。**
- `compiler.temp_dir`: 临时文件目录（空字符串表示使用系统默认）。**注意：当前实现始终通过 `tempfile::tempdir()` 使用系统默认临时目录。**

#### 日志配置
- `logging.level`: 日志级别（trace/debug/info/warn/error，默认：info）
- `logging.file_path`: 日志文件路径（默认：无，仅输出到控制台；当 `config.toml` 中显式配置 `logging.file_path = logs/server.log` 时会输出到文件）
- `logging.max_file_size_mb`: 单个日志文件最大大小（默认：10MB）。**注意：当前实现未启用按大小轮转。**
- `logging.max_backups`: 日志文件备份数量（默认：5）。**注意：当前实现未启用日志备份。**

## 使用示例

### 使用 curl 测试 API

#### 编译执行代码
```bash
# Python 示例
curl -X POST http://localhost:3000/api/compile \
  -H "Content-Type: application/json" \
  -d '{"language": "python", "code": "print(\"Hello, IDE!\")"}'

# 带输入的执行
curl -X POST http://localhost:3000/api/compile \
  -H "Content-Type: application/json" \
  -d '{"language": "python", "code": "print(input())", "input": "test"}'

# Java 示例
curl -X POST http://localhost:3000/api/compile \
  -H "Content-Type: application/json" \
  -d '{"language": "java", "code": "public class Main { public static void main(String[] args) { System.out.println(\"Hello\"); } }"}'
```

#### 代码静态分析
```bash
curl -X POST http://localhost:3000/api/analyze \
  -H "Content-Type: application/json" \
  -d '{"language": "python", "code": "x = 1\nprint(x)"}'
```

#### 获取统计信息
```bash
curl http://localhost:3000/stats
```

#### 健康检查
```bash
curl http://localhost:3000/health
```

### Web 界面使用

启动服务器后，在浏览器中访问 `http://127.0.0.1:3000` 即可使用内置的 Web 界面。

#### 功能特性
- **Monaco Editor 集成**：提供专业的代码编辑体验，支持语法高亮、自动补全、代码折叠等
- **多语言支持**：一键切换 Python、Java、C、C++、Rust 五种编程语言
- **实时代码分析**：编辑代码时（带防抖）自动调用 `/api/analyze` 进行静态分析，结果以 Monaco 诊断标记显示
- **标准输入支持**：可以为需要输入的程序提供测试数据
- **输出显示**：实时显示程序输出和错误信息
- **AI 代码助手**：内置对话式 AI 助手面板，**前端通过 HTTPS 直接调用第三方大模型 API（默认指向 `https://open.bigmodel.cn/api/paas/v4/chat/completions`，即智谱 AI 的 GLM 系列接口）**，使用前需要在 `frontend/index.html` 中配置对应的 API Key
- **状态栏**：显示当前语言和执行状态

#### 使用步骤
1. 选择编程语言
2. 在编辑器中编写代码
3. （可选）在输入框中提供测试数据
4. 点击 "Run" 按钮执行代码
5. 查看输出结果
6. 使用 AI 助手获取代码建议

## 核心模块说明

### main.rs
程序主入口，负责：
- 加载配置文件
- 初始化日志系统
- 创建并启动 HTTP 服务器
- 处理服务器关闭和统计信息输出

### server.rs
HTTP 服务器模块，提供：
- 基于 TCP 的简易 HTTP 服务器实现（不依赖第三方 Web 框架）
- 多线程并发请求处理（每连接一个 `std::thread`）
- API 端点路由：`/api/compile`、`/api/analyze`、`/stats`、`/health`、`/`
- 支持 JSON 与 `application/x-www-form-urlencoded` 两种请求体格式
- 请求解析和响应格式化（JSON + CORS 头）
- 执行统计和监控（按语言分类的成功/失败次数、平均耗时）

### compiler.rs
编译器模块，负责：
- 多语言代码编译和执行（Python 直解释执行；Java/C/C++/Rust 先编译后执行）
- 临时文件管理（通过 `tempfile` 库创建临时目录，并在结束时自动清理）
- 标准输入/输出处理（Python 通过子进程 stdin 写入；其他语言通过临时可执行文件运行）
- 编译和运行错误处理，统一为 `CodeResponse` 返回

### analyzer.rs
代码分析模块，提供：
- 多语言静态代码分析（Python/Java/C/C++/Rust）
- 各语言使用原生工具：
  - Python: `pylint --output-format=text --exit-zero`，不可用时回退到 `python -m py_compile` 语法检查
  - Java: `javac -Xlint:all -Xlint:-options`
  - C: `gcc -fsyntax-only -Wall -Wextra -pedantic -std=c11`，可用时附加 `cppcheck --quiet --enable=all`
  - C++: `g++ -fsyntax-only -Wall -Wextra -pedantic -std=c++17`，可用时附加 `cppcheck --quiet --enable=all`
  - Rust: `rustc --emit=metadata --edition=2021`，可用时附加 `cargo clippy --message-format=json`
- 输出标准化为 `AnalysisIssue`（severity/message/line/column/rule），再聚合成 `AnalysisResponse`

### models.rs
数据模型模块，定义：
- 支持的编程语言枚举 `Language`（含 `as_str`、`get_compiler`、`get_source_extension`、`get_display_name` 等方法）
- 请求结构体 `CodeRequest`
- 响应结构体 `CodeResponse`、`AnalysisResponse`、`AnalysisIssue`
- 编译结果 `CompileResult`
- 执行统计 `ExecutionStats`（含 `record_success/record_failure/get_average_time_ms` 等）

### config.rs
配置管理模块，负责：
- 配置文件加载和解析（自定义简化版 `key = value` 解析器，**不支持 TOML 嵌套数组与多文档**）
- 配置验证
- 默认配置管理
- 配置错误处理（`ConfigError` 枚举：FileNotFound/ParseError/InvalidValue/IoError）

### logging.rs
日志系统模块，提供：
- 多级别日志记录（Trace/Debug/Info/Warn/Error）
- 线程安全的日志输出（基于 `Arc<Mutex<...>>`）
- 控制台和文件双输出
- 日志格式化和时间戳（使用 `time` 库）
- **注意：当前实现未启用基于 `max_file_size_mb` / `max_backups` 的日志轮转，相关配置字段仅被读取存储**

### errors.rs
错误处理模块，定义：
- 统一的错误类型枚举
- 错误转换和显示
- Result 类型别名

## 测试

项目包含完整的单元测试：

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_language_from_str
cargo test test_compile_and_run_python

# 显示测试输出
cargo test -- --nocapture
```

测试覆盖：
- 语言解析和转换
- 编译器功能测试
- 代码执行测试
- 错误处理测试
- 配置管理测试
- 日志系统测试

## 许可证

MIT License

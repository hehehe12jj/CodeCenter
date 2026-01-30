# 实现计划：与 Claude Code 对话交互功能

## 需求概述

在 CodeCenter 页面上实现与 Claude Code 对话的交互功能：
- 用户在页面上输入消息
- 消息能够发送给 Claude Code 进程
- 接收并显示 Claude Code 的响应

## 现状分析

### 当前架构
- **被动监控**：仅通过监控日志文件检测会话状态
- **`send_message` 命令**：已定义但返回 `Err("此功能暂时未实现")`
- **`attach_to_session`**：返回 `can_send_input: false`
- **日志路径**：`~/.claude/projects/{encoded_path}/sessions/*.jsonl`

### 核心限制
Claude Code 进程通过 VSCode/IDE 启动，其 stdin/stdout 不直接可用。当前架构只能被动监控，无法主动发送输入。

## 实现方案：PTY 嵌入式终端

### 方案选择理由
1. **完全交互能力**：PTY 提供真正的交互式终端体验
2. **技术可行性**：macOS 支持良好
3. **用户体验**：与 Claude Code 内置终端一致

### 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                        前端 (React)                              │
├─────────────────────────────────────────────────────────────────┤
│  ChatPanel.tsx                                                   │
│  - 输入框组件                                                     │
│  - 消息列表显示                                                   │
│  - 终端输出显示 (xterm.js)                                        │
├─────────────────────────────────────────────────────────────────┤
│                        Tauri IPC                                 │
├─────────────────────────────────────────────────────────────────┤
│                    后端 (Rust)                                   │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  ClaudeProcessWrapper                                        ││
│  │  - PTY 进程管理                                              ││
│  │  - stdin/stdout 读写                                         ││
│  │  - 状态追踪                                                  ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  Tauri Commands                                             ││
│  │  - start_session(project_path) -> session_id                ││
│  │  - send_message(session_id, content)                        ││
│  │  - attach_to_session(session_id) -> connection_info         ││
│  │  - end_session(session_id)                                  ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## 文件变更

### 1. 新增文件

#### `src-tauri/src/wrapper/mod.rs`
**功能**：Claude Code 进程包装器，管理 PTY 进程

```rust
// 核心结构
pub struct ClaudeProcessWrapper {
    pid: u32,
    project_path: PathBuf,
    pty_master: PtyMaster,
    event_sender: mpsc::Sender<WrapperEvent>,
    running: Arc<RwLock<bool>>,
}

// 核心方法
impl ClaudeProcessWrapper {
    pub fn new(project_path: PathBuf) -> Result<Self>;
    pub async fn send_input(&self, content: &str) -> Result<()>;
    pub async fn read_output(&self) -> Result<Option<String>>;
    pub fn pid(&self) -> u32;
    pub async fn is_running(&self) -> bool;
    pub async fn terminate(&self) -> Result<()>;
}
```

#### `src-tauri/src/wrapper/pty.rs`
**功能**：PTY 相关工具函数

- `spawn_pty_command()`: 创建 PTY 命令
- `resize_pty()`: 调整终端大小
- `read_nonblocking()`: 非阻塞读取

#### `src-tauri/src/wrapper/events.rs`
**功能**：包装器事件定义

```rust
pub enum WrapperEvent {
    Output { session_id: String, content: String },
    Exited { session_id: String, exit_code: i32 },
    Error { session_id: String, message: String },
}
```

### 2. 修改文件

#### `src-tauri/src/state.rs`
新增状态字段：

```rust
pub struct AppState {
    // ... 现有字段
    pub process_wrapper: Arc<RwLock<ProcessWrapperManager>>,
}

pub struct ProcessWrapperManager {
    sessions: Arc<RwLock<HashMap<String, ClaudeProcessWrapper>>>,
    event_sender: mpsc::Sender<WrapperEvent>,
}
```

#### `src-tauri/src/commands/chat.rs`
重写命令实现：

```rust
#[tauri::command]
pub async fn send_message(
    session_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String>;

#[tauri::command]
pub async fn start_session(
    project_path: String,
    state: State<'_, AppState>,
) -> Result<SessionConnection, String>;

#[tauri::command]
pub async fn end_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String>;
```

#### `src-tauri/src/main.rs`
- 在 `setup()` 中初始化 `ProcessWrapperManager`
- 添加事件转发：从 `WrapperEvent` 到 Tauri 前端事件

#### `src-tauri/Cargo.toml`
新增依赖：

```toml
tokio-pty = "3"
nix = { version = "0.27", features = ["signal", "process", "termios"] }
```

### 3. 前端变更（参考）

文件：`src/components/ChatPanel.tsx`（前端团队实现）

- `invoke("start_session", { projectPath })` 启动会话
- `invoke("send_message", { sessionId, content })` 发送消息
- 监听 `wrapper:output` 事件显示输出
- 监听 `wrapper:exited` 事件处理会话结束

## 实现步骤

### 第一阶段：基础架构
1. 添加 `tokio-pty` 依赖
2. 创建 `wrapper` 模块目录
3. 实现 `ClaudeProcessWrapper` 核心结构
4. 实现 PTY 读写功能

### 第二阶段：命令集成
5. 在 `AppState` 中添加进程包装器管理
6. 重写 `send_message` 命令
7. 添加 `start_session` 和 `end_session` 命令
8. 在 `main.rs` 中注册新命令

### 第三阶段：事件系统
9. 实现 `WrapperEvent` 到前端事件的转发
10. 添加输出流式传输支持
11. 实现状态追踪

### 第四阶段：错误处理和优化
12. 添加完善的错误处理
13. 实现超时和重试机制
14. 添加单元测试

## 关键设计决策

### 1. PTY vs 文件注入
**选择 PTY**：
- Claude Code 支持交互式输入
- 可以正确处理特殊字符和ANSI转义码
- 用户体验更好

### 2. 会话管理
- 每个项目路径对应一个会话
- 支持并发多个会话
- 会话 ID 使用 UUID

### 3. 输出处理
- 使用 `tokio::process::Child` 的 stdout
- 非阻塞读取，避免阻塞事件循环
- 支持 ANSI 转义码解析（前端处理）

### 4. 状态同步
- 监控 PTY 进程状态
- 与现有的日志监控结合
- 状态变更事件推送到前端

## 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| PTY 依赖可能不稳定 | 添加完整的错误处理和回退机制 |
| Claude Code 启动参数变化 | 使用 `claude --help` 验证命令格式 |
| 进程权限问题 | 在用户主目录内操作 |
| 资源泄漏 | 添加超时机制和手动终止接口 |

## 验证方法

1. **单元测试**：
   - `cargo test` 通过
   - 测试 PTY 读写功能

2. **集成测试**：
   - 启动 Claude Code 会话
   - 发送消息并验证响应
   - 测试多个并发会话

3. **手动测试**：
   - 前端输入消息
   - 验证 Claude Code 接收并响应
   - 测试终端大小调整

## 依赖版本

```toml
tokio-pty = "3"
nix = { version = "0.27", features = ["signal", "process", "termios"] }
```

## 后续优化

1. 添加命令历史记录
2. 支持 ANSI 颜色解析
3. 实现 Tab 自动补全
4. 添加会话恢复功能

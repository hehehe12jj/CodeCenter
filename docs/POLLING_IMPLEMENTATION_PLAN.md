# 会话状态轮询机制实现计划

## 问题背景
当前应用使用事件驱动机制（文件系统监控 + Tauri 事件）来同步会话状态到前端，但该机制存在问题，导致前端无法正确显示新建或关闭的 Claude Code 会话。

## 解决方案
将事件驱动改为**轮询机制**：每 10 秒调用 `get_all_sessions` 获取完整会话列表，完全替换前端数据。

## 实现方案

### 阶段一：前端修改

#### 1. 修改 `src/features/sessions/store/useSessionStore.ts`

**删除内容：**
- 移除 `import { listen, UnlistenFn } from '@tauri-apps/api/event';`
- 移除 `initEventListeners` 方法
- 移除 `updateSessionStatus` 方法（不再需要，直接整体替换 sessions）
- 移除 `addSession` 方法（不再需要）

**添加内容：**
```typescript
// 轮询控制
pollingInterval: number; // 轮询间隔（毫秒）
pollingTimer: number | null; // setInterval 返回的 timer ID
startPolling: () => void;
stopPolling: () => void;
```

**实现逻辑：**
- `startPolling()`: 立即调用一次 `fetchSessions()`, 然后设置 10 秒间隔的定时器
- `stopPolling()`: 清除定时器

#### 2. 修改 `src/components/Dashboard.tsx`

**删除内容：**
- 移除 `initEventListeners` 从 store 解构
- 移除 `initEventListeners` 的 useEffect 调用

**修改内容：**
```typescript
useEffect(() => {
  // 初始加载并启动轮询
  fetchSessions().then(() => {
    setLastUpdated(new Date());
  });

  const { startPolling, stopPolling } = useSessionStore.getState();
  startPolling();

  // 组件卸载时停止轮询
  return () => {
    stopPolling();
  };
}, [fetchSessions]);
```

**保持不变的：**
- `handleRefresh` 函数（点击刷新按钮时立即调用 `fetchSessions`）
- 刷新按钮的 UI

### 阶段二：后端修改

#### 3. 修改 `src-tauri/src/main.rs`

**删除内容：**
- 移除 `forward_events_to_frontend` 函数（第 104-174 行）
- 移除 `serialize_status` 函数（第 177-186 行）
- 移除 `session_to_dto` 函数（第 189-202 行）
- 移除 `message_to_dto` 函数（第 205-214 行）
- 移除 `use tauri::{Emitter, Manager};` 中的 `Emitter`（如不再使用）

**简化 `setup` 函数：**
- 保留 AppState 初始化
- 保留 monitor 启动
- 移除事件转发相关的 spawn 代码

**修改后 setup 逻辑：**
```rust
.setup(|app| {
    tracing::info!("CodeAgent Dashboard starting...");

    // 同步初始化应用状态
    let state = tauri::async_runtime::block_on(async {
        AppState::init().await
    });

    match state {
        Ok(state) => {
            app.manage(state.clone());
            tracing::info!("AppState initialized successfully");

            // 启动会话监控器（用于 get_all_sessions 查询）
            tauri::async_runtime::spawn(async move {
                let monitor_started = {
                    let mut monitor = state.monitor.write().await;
                    tracing::info!("启动会话监控...");
                    match monitor.start().await {
                        Ok(_) => {
                            tracing::info!("Session monitor started successfully");
                            true
                        }
                        Err(e) => {
                            tracing::error!("Failed to start monitor: {}", e);
                            false
                        }
                    }
                };

                if !monitor_started {
                    tracing::error!("无法启动会话监控");
                }
            });
        }
        Err(e) => {
            tracing::error!("Failed to initialize AppState: {}", e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("AppState initialization failed: {}", e)
            )));
        }
    }

    Ok(())
})
```

### 阶段三：验证

#### 测试步骤：
1. 启动应用，观察控制台日志确认轮询正在执行
2. 在前端页面停留，观察 10 秒后是否自动刷新会话列表
3. 新建一个 Claude Code 会话，观察是否在 10 秒内出现在前端
4. 关闭一个 Claude Code 会话，观察是否在 10 内从前端消失或状态变更
5. 点击刷新按钮，观察是否立即重新加载（不等待 10 秒间隔）

#### 预期行为：
- 每 10 秒自动调用 `get_all_sessions` 命令
- 会话列表完全替换为最新数据
- 点击刷新按钮立即触发查询
- 控制台应看到 `[fetchSessions] 开始调用 get_all_sessions...` 日志每 10 秒输出一次

## 关键文件列表

| 文件 | 修改类型 | 说明 |
|------|---------|------|
| `src/features/sessions/store/useSessionStore.ts` | 修改 | 移除事件监听，添加轮询逻辑 |
| `src/components/Dashboard.tsx` | 修改 | 启动/停止轮询，移除事件监听调用 |
| `src-tauri/src/main.rs` | 修改 | 移除事件转发代码，简化 setup |

## 注意事项

1. **monitor 模块保留**：虽然移除了事件转发，但需要保留 monitor 的启动，因为 `get_all_sessions` 命令依赖 monitor 获取活跃会话
2. **错误处理**：轮询失败时不应停止轮询，应记录错误并继续下一次轮询
3. **内存泄漏**：确保组件卸载时正确调用 `stopPolling` 清除定时器
4. **竞态条件**：如果 `fetchSessions` 调用时间超过 10 秒，应防止重复请求

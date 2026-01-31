# CodeAgent Dashboard 前端 UI 优化计划

## 项目背景

CodeAgent Dashboard 是一个基于 Tauri 2.0 + React + TypeScript + TailwindCSS + Zustand 的桌面应用，用于统一管理多个 Claude Code 会话。

---

## 现状分析

### 当前代码结构

```
src/
├── components/
│   ├── Dashboard.tsx              # 主仪表盘
│   ├── SessionCard.tsx            # 旧版 SessionCard（基础版）
│   ├── StatusBar.tsx              # 状态栏
│   └── common/
│       ├── Button/
│       ├── LoadingSpinner/
│       └── StatusBadge/           # 状态徽章组件（已有）
├── features/sessions/
│   ├── components/
│   │   └── SessionCard/
│   │       └── SessionCard.tsx    # 增强版 SessionCard（当前在用）
│   └── store/useSessionStore.ts   # Zustand 状态管理
├── types/session.ts               # Session 类型定义
├── utils/formatters.ts            # 时间格式化工具
└── index.css                      # 全局样式
```

### 当前存在的问题

| 问题 | 现状描述 | 影响 |
|------|----------|------|
| 字体不专业 | 使用系统默认字体栈 (-apple-system, BlinkMacSystemFont 等) | 缺乏开发者工具的专业感 |
| 标题显示混乱 | 当前解析逻辑假设标题格式为 "folderName \| prompt"，但实际数据是路径格式如 "-Users-hejj" | 标题显示冗长、信息重复 |
| 路径显示重复 | projectName 和 title 显示重复内容 | 信息密度低 |
| 消息摘要未处理 | 直接显示原始 summary，包含 `<ide_selection>` 等标签 | 用户阅读体验差 |
| 缺少项目类型图标 | 所有项目使用相同图标 | 无法快速识别项目类型 |
| 字体配置不完整 | Tailwind 配置中没有自定义 fontFamily | 无法统一字体风格 |

---

## 优化计划

### Phase 1: 字体系统优化

**目标**：引入专业字体，提升开发者工具质感

**涉及文件**：
- `/Users/hejj/myproject/CodeCenter/tailwind.config.js` - 添加 fontFamily 配置
- `/Users/hejj/myproject/CodeCenter/index.html` - 添加 Google Fonts 链接
- `/Users/hejj/myproject/CodeCenter/src/index.css` - 更新默认字体

**具体改动**：
1. 在 `tailwind.config.js` 中添加：
   - `fontFamily.sans`: Inter 为主字体
   - `fontFamily.mono`: JetBrains Mono 为代码字体
2. 在 `index.html` 中引入 Google Fonts
3. 更新 `index.css` 中的 body 字体设置

---

### Phase 2: SessionCard 组件重构

**目标**：重新设计 SessionCard 的标题、路径、摘要显示逻辑

**涉及文件**：
- `/Users/hejj/myproject/CodeCenter/src/features/sessions/components/SessionCard/SessionCard.tsx` - 重构组件

**具体改动**：

#### 2.1 标题显示逻辑优化

当前逻辑：
```typescript
// 当前：假设 title 格式为 "folderName | prompt"
function parseTitle(title: string): { folderName: string; prompt: string }
```

新逻辑：
```typescript
// 新：根据 projectName 和 projectPath 智能显示
// 主标题：projectName（目录名）
// 副标题：简化路径（.../parent/projectName）
```

#### 2.2 新增项目类型图标

新建文件：`/Users/hejj/myproject/CodeCenter/src/utils/project-icons.tsx`

根据 projectPath 检测项目类型：
| 检测条件 | 图标 |
|----------|------|
| package.json + react 依赖 | React 图标 |
| package.json + vue 依赖 | Vue 图标 |
| package.json | Node.js 图标 |
| requirements.txt / pyproject.toml | Python 图标 |
| Cargo.toml | Rust 图标 |
| go.mod | Go 图标 |
| 其他 | 文件夹图标 |

#### 2.3 路径简化显示

新建工具函数：`/Users/hejj/myproject/CodeCenter/src/utils/path-utils.ts`

```typescript
function simplifyPath(fullPath: string): string {
  // 输入：/Users/hejj/projects/my-project
  // 输出：.../projects/my-project
}
```

---

### Phase 3: 消息摘要智能处理

**目标**：过滤日志标签，显示友好的摘要信息

**涉及文件**：
- `/Users/hejj/myproject/CodeCenter/src/utils/formatters.ts` - 添加摘要格式化函数
- `/Users/hejj/myproject/CodeCenter/src/features/sessions/components/SessionCard/SessionCard.tsx` - 使用新函数

**具体改动**：

在 `formatters.ts` 中添加：
```typescript
export function formatMessageSummary(rawContent: string | undefined): string {
  // 1. 移除 Claude Code 日志标签
  //    <ide_selection>, </ide_selection>, <function_calls> 等

  // 2. 智能识别场景
  //    - 用户选择代码 -> "👆 用户选择了代码片段"
  //    - 工具调用 -> "⚡ 正在调用工具函数"
  //    - AI 思考 -> "🤔 AI 正在思考..."
  //    - 错误 -> "⚠️ 遇到错误，需要处理"

  // 3. 空状态显示 -> "💬 会话刚开始，暂无消息"

  // 4. 截断处理（最大 80 字符）
}
```

---

### Phase 4: StatusBadge 位置优化

**目标**：将状态徽章移至卡片右上角，更醒目

**涉及文件**：
- `/Users/hejj/myproject/CodeCenter/src/features/sessions/components/SessionCard/SessionCard.tsx`

**当前布局**：状态徽章在标题右侧，与标题同行
**新布局**：状态徽章绝对定位在卡片右上角

---

### Phase 5: 空状态优化

**目标**：优化 "暂无消息摘要" 的显示

**涉及文件**：
- `/Users/hejj/myproject/CodeCenter/src/features/sessions/components/SessionCard/SessionCard.tsx`

**改动**：当 summary 为空或仅包含空白时，显示友好的引导文案：
```tsx
<div className="flex items-center gap-2 text-gray-400 text-sm">
  <MessageSquare className="w-4 h-4" />
  <span>会话刚开始，暂无消息</span>
</div>
```

---

### Phase 6: 卡片悬停效果增强

**目标**：添加更明显的悬停反馈

**涉及文件**：
- `/Users/hejj/myproject/CodeCenter/src/features/sessions/components/SessionCard/SessionCard.tsx`

**当前效果**：边框颜色变化、背景色变化
**新增效果**：
- 阴影增强：`hover:shadow-lg hover:shadow-black/20`
- 轻微上浮：`hover:-translate-y-0.5`

---

## 文件变更清单

| 序号 | 文件路径 | 变更类型 | 说明 |
|------|----------|----------|------|
| 1 | `/Users/hejj/myproject/CodeCenter/tailwind.config.js` | 修改 | 添加 fontFamily 配置 |
| 2 | `/Users/hejj/myproject/CodeCenter/index.html` | 修改 | 添加 Google Fonts 链接 |
| 3 | `/Users/hejj/myproject/CodeCenter/src/index.css` | 修改 | 更新默认字体为 Inter |
| 4 | `/Users/hejj/myproject/CodeCenter/src/utils/formatters.ts` | 修改 | 添加 formatMessageSummary 函数 |
| 5 | `/Users/hejj/myproject/CodeCenter/src/utils/path-utils.ts` | 新增 | 路径简化工具函数 |
| 6 | `/Users/hejj/myproject/CodeCenter/src/utils/project-icons.tsx` | 新增 | 项目类型图标组件 |
| 7 | `/Users/hejj/myproject/CodeCenter/src/features/sessions/components/SessionCard/SessionCard.tsx` | 修改 | 重构卡片布局和内容显示 |

---

## 验收标准

- [ ] 字体更换为 Inter（正文）和 JetBrains Mono（代码）
- [ ] Session Card Title 只显示 projectName，不显示冗长路径
- [ ] 添加项目类型图标（基于文件检测）
- [ ] 状态徽章移至卡片右上角
- [ ] 路径显示简化为 ".../parent/projectName" 格式
- [ ] 消息摘要过滤日志标签，显示友好文案
- [ ] 空状态显示引导性文案和图标
- [ ] 卡片悬停有阴影和上浮效果
- [ ] 整体视觉风格统一、专业

---

## 实施顺序

1. **Phase 1** → 字体系统优化（影响全局，先实施）
2. **Phase 3** → 消息摘要格式化（工具函数，无依赖）
3. **Phase 2, 4, 5, 6** → SessionCard 重构（可并行实施）

---

## 设计参考

- **Dashboard 布局**：参考 Vercel Dashboard、GitHub 仓库卡片
- **卡片设计**：圆角 12px、微妙阴影、悬停反馈
- **色彩方案**：保持现有深色/浅色主题支持
- **字体风格**：Inter 正文（400/500/600）、JetBrains Mono 代码

---

## 相关技术文档

- [TailwindCSS Typography](https://tailwindcss.com/docs/font-family)
- [Inter Font](https://rsms.me/inter/)
- [JetBrains Mono](https://www.jetbrains.com/lp/mono/)

---

*计划制定时间：2026-01-30*
*基于技术设计文档 v2.0*

# CodeAgent Dashboard 前端技术架构设计

## 文档信息

| 项目 | 内容 |
|------|------|
| 版本 | v1.0 |
| 日期 | 2026-01-29 |
| 状态 | 待评审 |
| 所属模块 | Presentation Layer |

---

## 1. 技术栈总览

### 1.1 核心技术选型

| 层级 | 技术 | 版本 | 选型理由 |
|------|------|------|----------|
| UI 框架 | React | 18.x | 生态成熟，组件化开发，Tauri 官方支持 |
| 语言 | TypeScript | 5.x | 类型安全，IDE 智能提示 |
| 样式 | TailwindCSS | 3.x | 原子化 CSS，开发效率高，主题切换方便 |
| 状态管理 | Zustand | 4.x | 轻量级，无样板代码，TypeScript 友好 |
| 路由 | React Router | 7.x | React 生态标准路由方案 |
| Tauri 集成 | @tauri-apps/api | 2.0.x | 官方前端 API，支持 Commands/Events |
| 终端模拟 | xterm.js | 5.x | 成熟的终端模拟器，支持添加插件 |
| 图标 | lucide-react | latest | 简洁现代，与 TailwindCSS 配合良好 |
| 虚拟列表 | tanstack/react-virtual | 3.x | 高性能虚拟滚动，处理长消息列表 |
| 日期处理 | date-fns | 3.x | 轻量级，按需引入 |
| HTTP 客户端 | axios | 1.x | 成熟的 HTTP 客户端 |

### 1.2 开发工具链

| 工具 | 用途 |
|------|------|
| Vite | 构建工具（替代 CRA，更快） |
| ESLint | 代码 lint |
| Prettier | 代码格式化 |
| Husky + lint-staged | Git hooks 管理 |
| Commitlint | 提交信息规范化 |

---

## 2. 项目结构设计

### 2.1 目录结构

```
src/
├── main.tsx                    # 应用入口
├── App.tsx                     # 根组件
├── index.html                  # HTML 模板
├── vite.config.ts              # Vite 配置
├── tailwind.config.js          # TailwindCSS 配置
├── postcss.config.js           # PostCSS 配置
│
├── assets/                     # 静态资源
│   ├── images/                 # 图片资源
│   ├── icons/                  # SVG 图标
│   └── fonts/                  # 字体文件
│
├── components/                 # 通用组件
│   ├── common/                 # 公共组件
│   │   ├── Button/
│   │   ├── Modal/
│   │   ├── Dropdown/
│   │   ├── Tooltip/
│   │   ├── Toast/
│   │   ├── Loading/
│   │   ├── EmptyState/
│   │   └── ErrorBoundary/
│   │
│   ├── layout/                 # 布局组件
│   │   ├── Header/
│   │   ├── Sidebar/
│   │   ├── MainLayout/
│   │   └── DashboardLayout/
│   │
│   └── feedback/               # 反馈组件
│       ├── StatusBadge/
│       ├── ProgressBar/
│       └── Notification/
│
├── features/                   # 功能模块（按领域组织）
│   ├── sessions/               # 会话功能
│   │   ├── components/         # 会话相关组件
│   │   │   ├── SessionList/
│   │   │   ├── SessionCard/
│   │   │   ├── SessionFilter/
│   │   │   └── SessionStats/
│   │   ├── hooks/              # 会话相关 Hooks
│   │   ├── services/           # 会话 API 服务
│   │   ├── store/              # 会话状态
│   │   ├── types/              # 会话类型定义
│   │   └── utils/              # 会话工具函数
│   │
│   ├── chat/                   # 对话功能
│   │   ├── components/
│   │   │   ├── ChatModal/
│   │   │   ├── MessageList/
│   │   │   ├── MessageInput/
│   │   │   └── CodeBlock/
│   │   ├── hooks/
│   │   ├── store/
│   │   ├── types/
│   │   └── utils/
│   │
│   └── projects/               # 项目功能
│       ├── components/
│       ├── hooks/
│       ├── store/
│       ├── types/
│       └── utils/
│
├── pages/                      # 页面组件
│   ├── Dashboard/
│   │   └── index.tsx
│   ├── Sessions/
│   │   ├── index.tsx
│   │   └── [id]/
│   │       └── index.tsx
│   ├── Projects/
│   │   └── index.tsx
│   ├── Settings/
│   │   └── index.tsx
│   └── NotFound/
│       └── index.tsx
│
├── hooks/                      # 全局 Hooks
│   ├── useSessions.ts
│   ├── useTheme.ts
│   ├── useTauri.ts
│   └── useKeyboard.ts
│
├── stores/                     # 全局状态
│   ├── sessionStore.ts
│   ├── projectStore.ts
│   ├── uiStore.ts
│   └── themeStore.ts
│
├── services/                   # API 服务
│   ├── api.ts                  # API 基础配置
│   ├── sessionService.ts
│   ├── projectService.ts
│   ├── configService.ts
│   └── eventService.ts
│
├── utils/                      # 工具函数
│   ├── helpers.ts
│   ├── formatters.ts
│   ├── validators.ts
│   ├── constants.ts
│   └── platform.ts
│
├── styles/                     # 全局样式
│   ├── globals.css
│   ├── variables.css
│   └── animations.css
│
├── types/                      # 全局类型定义
│   ├── session.ts
│   ├── project.ts
│   ├── message.ts
│   └── common.ts
│
├── lib/                        # 第三方库配置
│   ├── tauri.ts
│   ├── router.ts
│   └── analytics.ts
│
└── tests/                      # 测试配置
    ├── setup.ts
    └── mocks/
```

### 2.2 特征驱动结构（Feature-Based）

采用**特征驱动结构**组织代码，每个功能模块包含完整的垂直切片：

```
features/sessions/
├── components/         # 组件（仅在此功能内使用）
├── hooks/              # Hooks（仅在此功能内使用）
├── services/           # API 调用
├── store/              # 局部状态（可选）
├── types/              # 类型定义
├── utils/              # 工具函数
└── index.ts            # 导出
```

**原则**：
- 通用组件提取到 `components/common/`
- 页面组件在 `pages/`
- 跨功能共享的 Hooks 在 `hooks/`
- 跨功能共享的工具在 `utils/`

---

## 3. 组件架构设计

### 3.1 组件分类

```
┌─────────────────────────────────────────────────────────────┐
│                    Component Hierarchy                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                    Layout Components                 │    │
│  │  MainLayout ───► DashboardLayout                    │    │
│  └─────────────────────────────────────────────────────┘    │
│          │                    │                             │
│          ▼                    ▼                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                   Page Components                    │    │
│  │  DashboardPage  SessionsPage  SettingsPage          │    │
│  └─────────────────────────────────────────────────────┘    │
│          │                    │                             │
│          ▼                    ▼                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                 Feature Components                   │    │
│  │  SessionList  SessionCard  ChatModal  MessageList   │    │
│  └─────────────────────────────────────────────────────┘    │
│          │                    │                             │
│          ▼                    ▼                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                  Common Components                   │    │
│  │  Button  Modal  Dropdown  Toast  StatusBadge        │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 设计系统

#### 3.2.1 UI 设计规范

> **UI 设计规范来源**: [prototype.html](../../prototype.html)
>
> 本文档中的所有 UI 设计规范均基于 `prototype.html` 原型文件，该文件包含了完整的视觉设计、交互效果和组件样式。前端开发时应严格按照 prototype.html 中的设计实现，确保与原型视觉效果一致。

**核心设计原则**：
- **极简深色风格**：统一使用深色背景层级
- **玻璃态设计**：卡片和弹窗使用半透明 + backdrop-filter 效果
- **微交互**：hover、focus 等状态有平滑的过渡动画
- **状态可视化**：四种会话状态通过颜色和脉冲动画区分

**原型文件引用**：
| 设计元素 | 原型位置 | 说明 |
|---------|---------|------|
| 颜色系统 | `:root` CSS 变量 | 定义了完整的背景、文本、状态色 |
| 卡片样式 | `.session-card` 类 | 玻璃态卡片、圆角、边框、hover 效果 |
| 状态徽章 | `.status-badge` 类 | 四种状态的视觉样式 |
| 统计栏 | `.stats-bar` 类 | 头部统计信息展示 |
| 对话弹窗 | `.chat-modal` 类 | 消息列表和输入区域布局 |
| 动画效果 | `@keyframes` 动画 | fadeIn、scaleIn、pulse 等 |

#### 3.2.2 设计令牌（Design Tokens）

```typescript
// styles/tokens.ts

export const tokens = {
  colors: {
    // 状态色（与 prototype.html 一致）
    success: '#34d399',
    warning: '#fbbf24',
    error: '#f87171',
    info: '#60a5fa',
    // 中性色（与 prototype.html 一致）
    gray: {
      50: '#fafafa',
      100: '#f4f4f5',
      200: '#e4e4e7',
      300: '#d4d4d8',
      400: '#a1a1aa',
      500: '#71717a',
      600: '#52525b',
      700: '#3f3f46',
      800: '#27272a',
      900: '#18181b',
    },
    // 背景层级（与 prototype.html 一致）
    bg: {
      primary: '#0d0d0f',
      secondary: '#16161a',
      tertiary: '#1c1c22',
      card: 'rgba(28, 28, 34, 0.6)',
      glass: 'rgba(255, 255, 255, 0.03)',
    },
    // 边框颜色（与 prototype.html 一致）
    border: {
      default: 'rgba(255, 255, 255, 0.08)',
      hover: 'rgba(255, 255, 255, 0.12)',
    },
    // 文字颜色（与 prototype.html 一致）
    text: {
      primary: '#f5f5f7',
      secondary: 'rgba(255, 255, 255, 0.6)',
      tertiary: 'rgba(255, 255, 255, 0.4)',
    },
  },
  spacing: {
    xs: '4px',
    sm: '8px',
    md: '16px',
    lg: '24px',
    xl: '32px',
    '2xl': '48px',
  },
  borderRadius: {
    sm: '6px',
    md: '10px',
    lg: '16px',
    full: '9999px',
  },
  typography: {
    fontFamily: {
      sans: '-apple-system, BlinkMacSystemFont, SF Pro Display, Segoe UI, Roboto, sans-serif',
      mono: 'JetBrains Mono, Menlo, monospace',
    },
    fontSize: {
      xs: '12px',
      sm: '14px',
      base: '16px',
      lg: '18px',
      xl: '20px',
      '2xl': '24px',
      '3xl': '30px',
    },
  },
  shadows: {
    sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
    md: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
    lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1)',
    xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1)',
  },
  transitions: {
    fast: '150ms cubic-bezier(0.4, 0, 0.2, 1)',
    normal: '200ms cubic-bezier(0.4, 0, 0.2, 1)',
    slow: '300ms cubic-bezier(0.4, 0, 0.2, 1)',
  },
};
```

#### 3.2.3 TailwindCSS 配置

```javascript
// tailwind.config.js

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        bg: {
          primary: '#0d0d0f',
          secondary: '#16161a',
          tertiary: '#1c1c22',
          card: 'rgba(28, 28, 34, 0.6)',
          glass: 'rgba(255, 255, 255, 0.03)',
        },
        border: {
          DEFAULT: 'rgba(255, 255, 255, 0.08)',
          hover: 'rgba(255, 255, 255, 0.12)',
        },
        text: {
          primary: '#f5f5f7',
          secondary: 'rgba(255, 255, 255, 0.6)',
          tertiary: 'rgba(255, 255, 255, 0.4)',
        },
        status: {
          running: '#34d399',
          waiting: '#fbbf24',
          completed: '#60a5fa',
          blocked: '#f87171',
        },
      },
      backdropBlur: {
        xs: '2px',
      },
      animation: {
        'pulse-slow': 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'fade-in': 'fadeIn 300ms ease-out',
        'slide-down': 'slideDown 300ms ease-out',
        'slide-up': 'slideUp 300ms ease-out',
        'scale-in': 'scaleIn 300ms ease-out',
        'message-in': 'messageIn 300ms ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideDown: {
          '0%': { opacity: '0', transform: 'translateY(-10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        scaleIn: {
          '0%': { opacity: '0', transform: 'scale(0.95)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
        messageIn: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        pulse: {
          '0%, 100%': { opacity: '1', transform: 'scale(1)' },
          '50%': { opacity: '0.5', transform: 'scale(0.8)' },
        },
      },
    },
  },
  plugins: [],
};
```

### 3.3 核心组件设计

#### 3.3.1 会话卡片组件（SessionCard）

```tsx
// features/sessions/components/SessionCard/SessionCard.tsx

import React, { useMemo } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { cn } from '@/utils/cn';
import { useSessionActions } from '../../hooks/useSessionActions';
import { Session, SessionStatus } from '@/types/session';
import { Badge } from '@/components/common/Badge';
import { formatRelativeTime } from '@/utils/formatters';
import {
  Play,
  Clock,
  CheckCircle2,
  AlertCircle,
  MoreVertical,
  ExternalLink,
  Archive,
  Trash2,
} from 'lucide-react';

interface SessionCardProps {
  session: Session;
  isSelected?: boolean;
  onSelect?: (session: Session) => void;
  className?: string;
}

const statusConfig = {
  running: {
    icon: Play,
    color: 'text-status-running',
    bgColor: 'bg-status-running/20',
    pulse: true,
    label: '运行中',
  },
  waiting_input: {
    icon: Clock,
    color: 'text-status-waiting',
    bgColor: 'bg-status-waiting/20',
    pulse: false,
    label: '等待输入',
  },
  completed: {
    icon: CheckCircle2,
    color: 'text-status-completed',
    bgColor: 'bg-status-completed/20',
    pulse: false,
    label: '已完成',
  },
  blocked: {
    icon: AlertCircle,
    color: 'text-status-blocked',
    bgColor: 'bg-status-blocked/20',
    pulse: true,
    label: '执行阻塞',
  },
};

export const SessionCard: React.FC<SessionCardProps> = ({
  session,
  isSelected = false,
  onSelect,
  className,
}) => {
  const navigate = useNavigate();
  const { openTerminal, markCompleted, archiveSession } = useSessionActions();

  const status = statusConfig[session.status];
  const StatusIcon = status.icon;

  const formattedTime = useMemo(() => {
    return formatRelativeTime(session.lastActiveAt);
  }, [session.lastActiveAt]);

  const handleCardClick = () => {
    onSelect?.(session);
    navigate({ to: '/sessions/$sessionId', params: { sessionId: session.id } });
  };

  const handleOpenTerminal = (e: React.MouseEvent) => {
    e.stopPropagation();
    openTerminal(session.projectPath);
  };

  const handleMarkCompleted = (e: React.MouseEvent) => {
    e.stopPropagation();
    markCompleted(session.id);
  };

  return (
    <div
      className={cn(
        'group relative overflow-hidden rounded-xl border border-border bg-bg-card/60',
        'backdrop-blur-xl transition-all duration-200',
        'hover:border-border-hover hover:bg-bg-tertiary/80',
        'cursor-pointer',
        isSelected && 'ring-2 ring-primary-500/50 border-primary-500',
        className
      )}
      onClick={handleCardClick}
    >
      {/* 状态指示灯 */}
      <div className="absolute left-0 top-0 bottom-0 w-1">
        <div
          className={cn(
            'absolute inset-y-0 left-0 w-full bg-status-running',
            status.bgColor,
            status.pulse && 'animate-pulse-slow'
          )}
        />
      </div>

      <div className="p-4 pl-5">
        {/* 头部 */}
        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 min-w-0">
            <h3 className="font-medium text-text-primary truncate">
              {session.title}
            </h3>
            <p className="text-sm text-text-secondary truncate mt-0.5">
              {session.projectName}
            </p>
          </div>

          {/* 状态徽章 */}
          <Badge
            variant="status"
            status={session.status}
            icon={StatusIcon}
            pulse={status.pulse}
          >
            {status.label}
          </Badge>
        </div>

        {/* 摘要 */}
        {session.summary && (
          <p className="mt-3 text-sm text-text-tertiary line-clamp-2">
            {session.summary}
          </p>
        )}

        {/* 元信息 */}
        <div className="mt-4 flex items-center justify-between text-xs text-text-tertiary">
          <div className="flex items-center gap-4">
            <span>开始于 {formatRelativeTime(session.createdAt)}</span>
            <span>活跃于 {formattedTime}</span>
          </div>

          {/* 操作按钮 */}
          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              onClick={handleOpenTerminal}
              className="p-1.5 rounded-md hover:bg-white/10 transition-colors"
              title="打开终端"
            >
              <ExternalLink className="w-4 h-4" />
            </button>
            <button
              onClick={handleMarkCompleted}
              className="p-1.5 rounded-md hover:bg-white/10 transition-colors"
              title="标记完成"
            >
              <CheckCircle2 className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* 悬停效果 */}
      <div className="absolute inset-0 pointer-events-none ring-1 ring-inset ring-white/5 rounded-xl" />
    </div>
  );
};
```

#### 3.3.2 状态徽章组件（StatusBadge）

```tsx
// components/feedback/StatusBadge/StatusBadge.tsx

import React from 'react';
import { cn } from '@/utils/cn';
import { SessionStatus } from '@/types/session';
import { LucideIcon } from 'lucide-react';

interface StatusBadgeProps {
  status: SessionStatus;
  icon?: LucideIcon;
  pulse?: boolean;
  children?: React.ReactNode;
  className?: string;
}

const statusStyles: Record<SessionStatus, string> = {
  running: 'bg-status-running/20 text-status-running border-status-running/30',
  waiting_input: 'bg-status-waiting/20 text-status-waiting border-status-waiting/30',
  completed: 'bg-status-completed/20 text-status-completed border-status-completed/30',
  blocked: 'bg-status-blocked/20 text-status-blocked border-status-blocked/30',
  unknown: 'bg-gray-500/20 text-gray-400 border-gray-500/30',
};

export const StatusBadge: React.FC<StatusBadgeProps> = ({
  status,
  icon: Icon,
  pulse = false,
  children,
  className,
}) => {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium',
        'border',
        statusStyles[status],
        pulse && 'animate-pulse-slow',
        className
      )}
    >
      {Icon && <Icon className="w-3 h-3" />}
      {children}
    </span>
  );
};
```

#### 3.3.3 对话弹窗组件（ChatModal）

```tsx
// features/chat/components/ChatModal/ChatModal.tsx

import React, { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { cn } from '@/utils/cn';
import { useSessionMessages } from '../../hooks/useSessionMessages';
import { MessageList } from '../MessageList';
import { MessageInput } from '../MessageInput';
import { Session } from '@/types/session';
import { X, Maximize2, Minimize2, Loader2 } from 'lucide-react';

interface ChatModalProps {
  session: Session;
  isOpen: boolean;
  onClose: () => void;
  className?: string;
}

export const ChatModal: React.FC<ChatModalProps> = ({
  session,
  isOpen,
  onClose,
  className,
}) => {
  const [isMaximized, setIsMaximized] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const modalRef = useRef<HTMLDivElement>(null);

  const {
    messages,
    isLoading: messagesLoading,
    sendMessage,
    loadMore,
    hasMore,
  } = useSessionMessages(session.id, 30);

  // 处理 ESC 键关闭
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };

    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
      document.body.style.overflow = 'hidden';
    }

    return () => {
      document.removeEventListener('keydown', handleEscape);
      document.body.style.overflow = '';
    };
  }, [isOpen, onClose]);

  // 点击遮罩层关闭
  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  if (!isOpen) return null;

  const modalContent = (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      onClick={handleOverlayClick}
    >
      {/* 遮罩层 */}
      <div className="absolute inset-0 bg-black/60 backdrop-blur-sm animate-fade-in" />

      {/* 模态框 */}
      <div
        ref={modalRef}
        className={cn(
          'relative bg-bg-secondary rounded-2xl shadow-2xl',
          'flex flex-col overflow-hidden',
          'animate-scale-in',
          'transition-all duration-200',
          isMaximized
            ? 'inset-4'
            : 'w-[800px] h-[600px]',
          className
        )}
      >
        {/* 头部 */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <div className="flex items-center gap-3">
            <div className="w-3 h-3 rounded-full bg-status-running animate-pulse" />
            <h2 className="font-medium text-text-primary">
              {session.title}
            </h2>
            <span className="text-xs text-text-tertiary px-2 py-0.5 bg-bg-tertiary rounded">
              {session.projectName}
            </span>
          </div>

          <div className="flex items-center gap-1">
            <button
              onClick={() => setIsMaximized(!isMaximized)}
              className="p-1.5 rounded-md hover:bg-white/10 transition-colors"
            >
              {isMaximized ? (
                <Minimize2 className="w-4 h-4" />
              ) : (
                <Maximize2 className="w-4 h-4" />
              )}
            </button>
            <button
              onClick={onClose}
              className="p-1.5 rounded-md hover:bg-white/10 transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* 消息列表 */}
        <div className="flex-1 overflow-hidden">
          {messagesLoading ? (
            <div className="flex items-center justify-center h-full">
              <Loader2 className="w-6 h-6 animate-spin text-text-tertiary" />
            </div>
          ) : (
            <MessageList
              messages={messages}
              onLoadMore={loadMore}
              hasMore={hasMore}
            />
          )}
        </div>

        {/* 输入框 */}
        <div className="border-t border-border p-4">
          <MessageInput
            onSend={async (content) => {
              setIsLoading(true);
              await sendMessage(content);
              setIsLoading(false);
            }}
            disabled={isLoading}
            placeholder="输入消息... (Enter 发送，Shift+Enter 换行)"
          />
        </div>
      </div>
    </div>
  );

  // 使用 Portal 渲染到 body
  return createPortal(modalContent, document.body);
};
```

#### 3.3.4 消息列表组件（MessageList - 带虚拟滚动）

```tsx
// features/chat/components/MessageList/MessageList.tsx

import React, { useRef, useCallback } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { cn } from '@/utils/cn';
import { Message } from '@/types/message';
import { MessageItem } from '../MessageItem';
import { Loader2 } from 'lucide-react';

interface MessageListProps {
  messages: Message[];
  onLoadMore?: () => void;
  hasMore?: boolean;
  className?: string;
}

export const MessageList: React.FC<MessageListProps> = ({
  messages,
  onLoadMore,
  hasMore = false,
  className,
}) => {
  const parentRef = useRef<HTMLDivElement>(null);

  // 虚拟滚动配置
  const rowVirtualizer = useVirtualizer({
    count: messages.length + (hasMore ? 1 : 0), // 预留空间给加载更多按钮
    getScrollElement: () => parentRef.current,
    estimateSize: () => 80, // 预估每条消息高度
    overscan: 5,
  });

  // 渲染消息项
  const renderMessage = useCallback(
    (index: number) => {
      // 如果是加载更多按钮
      if (hasMore && index === 0) {
        return (
          <div
            key="load-more"
            className="flex justify-center py-4"
            style={{
              height: '60px',
            }}
          >
            <button
              onClick={onLoadMore}
              className="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
            >
              加载更多消息
            </button>
          </div>
        );
      }

      // 真实消息索引（偏移 1）
      const messageIndex = hasMore ? index - 1 : index;
      const message = messages[messageIndex];

      return (
        <div
          key={message.id}
          className="px-4 py-2"
          style={{
            height: 'auto', // 让内容决定高度
          }}
        >
          <MessageItem message={message} />
        </div>
      );
    },
    [messages, hasMore, onLoadMore]
  );

  return (
    <div
      ref={parentRef}
      className={cn(
        'h-full overflow-y-auto scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent',
        className
      )}
    >
      <div
        style={{
          height: `${rowVirtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {rowVirtualizer.getVirtualItems().map((virtualRow) => (
          <div
            key={virtualRow.key}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              transform: `translateY(${virtualRow.start}px)`,
            }}
          >
            {renderMessage(virtualRow.index)}
          </div>
        ))}
      </div>
    </div>
  );
};
```

---

## 4. 状态管理设计

### 4.1 状态分类

```
┌─────────────────────────────────────────────────────────────┐
│                    State Categories                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Server State (远程状态)                 │    │
│  │  • 会话列表      • 消息历史    • 项目列表            │    │
│  │  • 配置数据      • 用户信息                          │    │
│  │  管理方式: TanStack Query (React Query)             │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Client State (本地状态)                 │    │
│  │  • UI 状态        • 展开/折叠状态                    │    │
│  │  • 主题设置       • 快捷键绑定                       │    │
│  │  管理方式: Zustand                                  │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Navigation State (路由状态)             │    │
│  │  • 当前路径      • 路由参数    • 查询参数            │    │
│  │  • 历史记录      • 滚动位置                          │    │
│  │  管理方式: React Router                             │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Zustand 状态设计

#### 4.2.1 UI 状态存储

```typescript
// stores/uiStore.ts

import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

interface UIState {
  // 侧边栏状态
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;

  // 模态框状态
  activeModal: string | null;
  modalProps: Record<string, unknown>;
  openModal: (modalId: string, props?: Record<string, unknown>) => void;
  closeModal: () => void;

  // Toast 通知
  toasts: Toast[];
  addToast: (toast: Omit<Toast, 'id'>) => void;
  removeToast: (id: string) => void;

  // 搜索状态
  searchQuery: string;
  searchActive: boolean;
  setSearchQuery: (query: string) => void;
  setSearchActive: (active: boolean) => void;

  // 主题
  theme: 'dark' | 'light';
  setTheme: (theme: 'dark' | 'light') => void;
}

interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
}

export const useUIStore = create<UIState>()(
  devtools(
    persist(
      (set, get) => ({
        // 侧边栏
        sidebarCollapsed: false,
        toggleSidebar: () =>
          set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

        // 模态框
        activeModal: null,
        modalProps: {},
        openModal: (modalId, props = {}) =>
          set({ activeModal: modalId, modalProps: props }),
        closeModal: () => set({ activeModal: null, modalProps: {} }),

        // Toast
        toasts: [],
        addToast: (toast) => {
          const id = crypto.randomUUID();
          set((state) => ({
            toasts: [...state.toasts, { ...toast, id }],
          }));

          // 自动移除
          if (toast.duration !== 0) {
            setTimeout(() => {
              get().removeToast(id);
            }, toast.duration || 5000);
          }
        },
        removeToast: (id) =>
          set((state) => ({
            toasts: state.toasts.filter((t) => t.id !== id),
          })),

        // 搜索
        searchQuery: '',
        searchActive: false,
        setSearchQuery: (query) => set({ searchQuery: query }),
        setSearchActive: (active) => set({ searchActive: active }),

        // 主题
        theme: 'dark',
        setTheme: (theme) => set({ theme }),
      }),
      {
        name: 'ui-storage',
        partialize: (state) => ({
          sidebarCollapsed: state.sidebarCollapsed,
          theme: state.theme,
        }),
      }
    ),
    { name: 'UIStore' }
  )
);
```

#### 4.2.2 会话状态存储

```typescript
// stores/sessionStore.ts

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { Session, SessionStatus } from '@/types/session';

interface SessionState {
  // 数据
  sessions: Session[];
  selectedSessionId: string | null;
  loading: boolean;
  error: string | null;

  // 操作
  fetchSessions: () => Promise<void>;
  selectSession: (id: string | null) => void;
  updateSessionStatus: (id: string, status: SessionStatus) => void;
  markCompleted: (id: string) => Promise<void>;
  archiveSession: (id: string) => Promise<void>;

  // 事件监听
  initEventListeners: () => () => void;
}

export const useSessionStore = create<SessionState>()(
  devtools(
    (set, get) => ({
      // 初始状态
      sessions: [],
      selectedSessionId: null,
      loading: false,
      error: null,

      // 获取所有会话
      fetchSessions: async () => {
        set({ loading: true, error: null });
        try {
          const sessions = await invoke<Session[]>('get_all_sessions');
          set({ sessions, loading: false });
        } catch (error) {
          set({ error: String(error), loading: false });
        }
      },

      // 选择会话
      selectSession: (id) => set({ selectedSessionId: id }),

      // 更新会话状态
      updateSessionStatus: (id, status) =>
        set((state) => ({
          sessions: state.sessions.map((s) =>
            s.id === id ? { ...s, status } : s
          ),
        })),

      // 标记完成
      markCompleted: async (id) => {
        try {
          await invoke('mark_session_completed', { id });
          set((state) => ({
            sessions: state.sessions.map((s) =>
              s.id === id ? { ...s, status: 'completed' as SessionStatus } : s
            ),
          }));
        } catch (error) {
          get().fetchSessions(); // 刷新以恢复
          throw error;
        }
      },

      // 归档会话
      archiveSession: async (id) => {
        try {
          await invoke('archive_session', { id });
          set((state) => ({
            sessions: state.sessions.map((s) =>
              s.id === id ? { ...s, isArchived: true } : s
            ),
          }));
        } catch (error) {
          get().fetchSessions();
          throw error;
        }
      },

      // 初始化事件监听
      initEventListeners: () => {
        // 监听会话状态变更
        const statusUnlisten = listen<{
          sessionId: string;
          newStatus: SessionStatus;
        }>('session:status-changed', (event) => {
          get().updateSessionStatus(
            event.payload.sessionId,
            event.payload.newStatus
          );
        });

        // 监听会话发现
        const discoveredUnlisten = listen<Session>('session:discovered', (event) => {
          set((state) => ({
            sessions: [event.payload, ...state.sessions],
          }));
        });

        // 返回清理函数
        return () => {
          statusUnlisten.then((fn) => fn());
          discoveredUnlisten.then((fn) => fn());
        };
      },
    }),
    { name: 'SessionStore' }
  )
);
```

### 4.3 TanStack Query 数据获取

```typescript
// lib/queryClient.ts

import { QueryClient } from '@tanstack/react-query';

// 创建 QueryClient 实例
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30 * 1000, // 30 秒内数据视为新鲜
      gcTime: 5 * 60 * 1000, // 5 分钟后垃圾回收
      retry: 1,
      refetchOnWindowFocus: false,
    },
    mutations: {
      retry: 0,
    },
  },
});

// hooks/useSession.ts

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { Session } from '@/types/session';

export function useSessions() {
  return useQuery({
    queryKey: ['sessions'],
    queryFn: () => invoke<Session[]>('get_all_sessions'),
  });
}

export function useSessionDetail(sessionId: string, messageLimit = 30) {
  return useQuery({
    queryKey: ['session', sessionId, messageLimit],
    queryFn: () =>
      invoke('get_session_detail', { id: sessionId, messageLimit }),
    enabled: !!sessionId,
  });
}

export function useSessionMessages(sessionId: string, limit = 30) {
  const queryClient = useQueryClient();

  const { data: messages, isLoading } = useQuery({
    queryKey: ['session', sessionId, 'messages', limit],
    queryFn: () =>
      invoke('get_session_messages', { sessionId, limit }),
    staleTime: 10 * 1000, // 消息相对稳定
  });

  const loadMore = async () => {
    const currentMessages = queryClient.getQueryData<Message[]>([
      'session',
      sessionId,
      'messages',
      limit,
    ]);

    if (!currentMessages?.length) return;

    const beforeId = currentMessages[0]?.id;
    const moreMessages = await invoke<Message[]>('load_more_messages', {
      sessionId,
      beforeId,
      limit,
    });

    // 合并并去重
    queryClient.setQueryData(
      ['session', sessionId, 'messages', limit],
      [...moreMessages, ...currentMessages]
    );
  };

  return { messages, isLoading, loadMore };
}

export function useSendMessage() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ sessionId, content }: { sessionId: string; content: string }) =>
      invoke('send_message', { sessionId, content }),
    onSuccess: (_, { sessionId }) => {
      // 使相关查询失效
      queryClient.invalidateQueries({
        queryKey: ['session', sessionId, 'messages'],
      });
      queryClient.invalidateQueries({
        queryKey: ['session', sessionId],
      });
    },
  });
}
```

---

## 5. Tauri 集成设计

### 5.1 Commands 封装

```typescript
// lib/tauri/commands.ts

import { invoke } from '@tauri-apps/api/tauri';
import { Session, SessionDetail, Project, AppConfig } from '@/types';

// 类型化调用函数
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}

// 会话相关 Commands
export const sessionCommands = {
  getAllSessions: () => call<Session[]>('get_all_sessions'),

  getSessionDetail: (id: string, messageLimit?: number) =>
    call<SessionDetail>('get_session_detail', { id, messageLimit }),

  loadMoreMessages: (sessionId: string, beforeId: string, limit: number) =>
    call('load_more_messages', { sessionId, beforeId, limit }),

  markSessionCompleted: (id: string) =>
    call('mark_session_completed', { id }),

  archiveSession: (id: string) =>
    call('archive_session', { id }),

  unarchiveSession: (id: string) =>
    call('unarchive_session', { id }),
};

// 对话相关 Commands
export const chatCommands = {
  attachToSession: (sessionId: string) =>
    call('attach_to_session', { sessionId }),

  sendMessage: (sessionId: string, content: string) =>
    call('send_message', { sessionId, content }),

  detachFromSession: (sessionId: string) =>
    call('detach_from_session', { sessionId }),
};

// 系统 Commands
export const systemCommands = {
  openTerminal: (projectPath: string) =>
    call('open_terminal', { projectPath }),

  getAllProjects: () => call<Project[]>('get_all_projects'),

  refreshStatus: () => call('refresh_status'),

  getConfig: () => call<AppConfig>('get_config'),

  updateConfig: (config: Partial<AppConfig>) =>
    call('update_config', { config }),
};
```

### 5.2 Events 封装

```typescript
// lib/tauri/events.ts

import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';

// 事件类型定义
interface SessionEvents {
  'session:discovered': { session: Session };
  'session:status-changed': {
    sessionId: string;
    oldStatus: string;
    newStatus: string;
  };
  'session:updated': {
    sessionId: string;
    changes: Partial<Session>;
  };
  'session:removed': { sessionId: string };
}

interface MessageEvents {
  'message:received': {
    sessionId: string;
    message: Message;
  };
  'message:updated': {
    sessionId: string;
    messageId: string;
    content: string;
  };
}

interface SystemEvents {
  'monitor:error': { error: string; details?: string };
  'status:refreshed': { timestamp: string };
}

// 通用事件 Hook
export function useTauriEvent<K extends keyof SessionEvents>(
  event: K,
  handler: (payload: SessionEvents[K]) => void
): UnlistenFn {
  useEffect(() => {
    const unlisten = listen<SessionEvents[typeof event]>(event, (e) => {
      handler(e.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [event, handler]);
}

// 状态变更 Hook
export function useSessionStatusChange(
  onChange: (sessionId: string, newStatus: string) => void
) {
  return useTauriEvent('session:status-changed', (payload) => {
    onChange(payload.sessionId, payload.newStatus);
  });
}

// 新消息 Hook
export function useNewMessage(
  onMessage: (sessionId: string, message: Message) => void
) {
  return useTauriEvent('message:received', (payload) => {
    onMessage(payload.sessionId, payload.message);
  });
}
```

---

## 6. 性能优化策略

### 6.1 渲染优化

```typescript
// 组件级别优化示例

// 1. React.memo 包装
import { memo, useCallback, useMemo } from 'react';

// 纯组件使用 memo
export const SessionCard = memo<SessionCardProps>(({ session, onSelect }) => {
  // 回调使用 useCallback
  const handleClick = useCallback(() => {
    onSelect?.(session);
  }, [session, onSelect]);

  // 计算值使用 useMemo
  const statusColor = useMemo(() => {
    return statusConfig[session.status].color;
  }, [session.status]);

  return <div onClick={handleClick}>{/* ... */}</div>;
});

// 2. 虚拟列表优化长列表
import { useVirtualizer } from '@tanstack/react-virtual';

// 3. 延迟加载非关键组件
const HeavyComponent = lazy(() => import('./HeavyComponent'));

// 4. 代码分割
const Dashboard = lazy(() => import('./pages/Dashboard'));
```

### 6.2 状态优化

```typescript
// 避免不必要的状态更新
function useOptimisticUpdate<T>(
  queryKey: readonly unknown[],
  updateFn: (oldData: T) => T
) {
  const queryClient = useQueryClient();

  return useCallback(
    (newData: T | ((old: T) => T)) => {
      queryClient.setQueryData(queryKey, (old: T | undefined) => {
        if (typeof newData === 'function') {
          return (newData as (old: T) => T)(old || ({} as T));
        }
        return updateFn(newData);
      });
    },
    [queryClient, queryKey, updateFn]
  );
}
```

### 6.3 资源优化

```typescript
// 图片懒加载
function useLazyImage(src: string) {
  const [isLoaded, setIsLoaded] = useState(false);
  const imgRef = useRef<HTMLImageElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          const img = imgRef.current;
          if (img && img.dataset.src) {
            img.src = img.dataset.src;
            img.onload = () => setIsLoaded(true);
            observer.unobserve(img);
          }
        }
      },
      { rootMargin: '100px' }
    );

    if (imgRef.current) {
      observer.observe(imgRef.current);
    }

    return () => observer.disconnect();
  }, [src]);

  return { imgRef, isLoaded };
}
```

---

## 7. 测试策略

### 7.1 测试分层

```
┌─────────────────────────────────────────────────────────────┐
│                    Testing Pyramid                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                    ┌───────────┐                            │
│                    │   E2E     │   少量，高价值             │
│                    │   Tests   │   关键用户流程             │
│                    └─────┬─────┘                            │
│                          │                                  │
│              ┌───────────┴───────────┐                      │
│              │     Integration       │   中等数量           │
│              │       Tests           │   组件集成           │
│              └───────────┬───────────┘                      │
│                          │                                  │
│      ┌───────────────────┴───────────────────┐              │
│      │           Unit Tests                  │   大量，快速  │
│      │     (Vitest + React Testing Library)  │   工具函数    │
│      └───────────────────────────────────────┘   组件逻辑    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 测试配置

```typescript
// vite.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
  plugins: [react(), tsconfigPaths()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/tests/setup.ts'],
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
    },
  },
});
```

### 7.3 测试示例

```typescript
// tests/unit/components/SessionCard.test.tsx

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SessionCard } from '@/features/sessions/components/SessionCard';
import { MemoryRouter } from 'react-router';

const mockSession = {
  id: 'test-id',
  title: '测试会话',
  projectName: 'test-project',
  projectPath: '/path/to/project',
  status: 'running',
  createdAt: new Date().toISOString(),
  lastActiveAt: new Date().toISOString(),
  summary: '这是一个测试会话',
  agentType: 'claude',
  isArchived: false,
};

describe('SessionCard', () => {
  it('渲染会话基本信息', () => {
    render(
      <MemoryRouter>
        <SessionCard session={mockSession} />
      </MemoryRouter>
    );

    expect(screen.getByText('测试会话')).toBeInTheDocument();
    expect(screen.getByText('test-project')).toBeInTheDocument();
  });

  it('显示正确的状态徽章', () => {
    render(
      <MemoryRouter>
        <SessionCard session={mockSession} />
      </MemoryRouter>
    );

    expect(screen.getByText('运行中')).toBeInTheDocument();
  });

  it('点击触发选择回调', () => {
    const onSelect = vi.fn();
    render(
      <MemoryRouter>
        <SessionCard session={mockSession} onSelect={onSelect} />
      </MemoryRouter>
    );

    fireEvent.click(screen.getByRole('article'));
    expect(onSelect).toHaveBeenCalledWith(mockSession);
  });
});
```

```typescript
// tests/integration/sessions.test.ts

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { playwright } from 'playwright';

describe('Sessions E2E', () => {
  let browser: any;

  beforeAll(async () => {
    browser = await playwright.chromium.launch();
  });

  afterAll(async () => {
    await browser.close();
  });

  it('显示会话列表', async () => {
    const page = await browser.newPage();
    await page.goto('http://localhost:5173');

    // 等待加载
    await page.waitForSelector('[data-testid="session-list"]');

    // 验证会话卡片存在
    const cards = await page.locator('[data-testid="session-card"]').count();
    expect(cards).toBeGreaterThan(0);
  });

  it('点击会话打开对话弹窗', async () => {
    const page = await browser.newPage();
    await page.goto('http://localhost:5173');

    // 点击第一个会话卡片
    await page.locator('[data-testid="session-card"]').first().click();

    // 验证对话弹窗出现
    await page.waitForSelector('[data-testid="chat-modal"]');
    expect(page.locator('[data-testid="message-input"]')).toBeVisible();
  });
});
```

---

## 8. 构建与部署

### 8.1 Vite 配置

```typescript
// vite.config.ts

import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tsconfigPaths from 'vite-tsconfig-paths';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react(), tsconfigPaths()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  build: {
    target: 'esnext',
    minify: 'esbuild',
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom', 'react-router-dom'],
          ui: ['lucide-react', 'clsx', 'tailwind-merge'],
          query: ['@tanstack/react-query'],
        },
      },
    },
  },
});
```

### 8.2 CI/CD 配置

```yaml
# .github/workflows/frontend.yml
name: Frontend CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  lint-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install Dependencies
        run: npm ci

      - name: Lint
        run: npm run lint

      - name: Type Check
        run: npm run type-check

      - name: Run Tests
        run: npm run test:coverage

  build:
    needs: lint-and-test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install Dependencies
        run: npm ci

      - name: Build
        run: npm run build

      - name: Upload Build
        uses: actions/upload-artifact@v4
        with:
          name: frontend-build
          path: dist/
```

---

## 9. 代码规范

### 9.1 组件规范

```typescript
// 组件模板

// 1. 导入顺序
import React from 'react';
import { useMemo, useCallback } from 'react';

// 第三方库
import { useQuery } from '@tanstack/react-query';

// 内部组件
import { Button } from '@/components/common/Button';
import { Modal } from '@/components/common/Modal';

// hooks
import { useSessionActions } from '../../hooks/useSessionActions';

// 类型
import { Session } from '@/types/session';

// 工具
import { cn } from '@/utils/cn';
import { formatTime } from '@/utils/formatters';

// 2. 组件结构
interface ComponentNameProps {
  // 类型定义
  className?: string;
}

// 3. 组件实现
export const ComponentName: React.FC<ComponentNameProps> = ({
  className,
}) => {
  // hooks
  const {} = useSomething();

  // 计算值
  const value = useMemo(() => {
    // 计算逻辑
  }, [deps]);

  // 回调
  const handleClick = useCallback(() => {
    // 处理逻辑
  }, [deps]);

  // 渲染
  return (
    <div className={cn('', className)}>
      {/* 内容 */}
    </div>
  );
};
```

### 9.2 提交规范

```
feat: 新功能
fix: Bug 修复
docs: 文档更新
style: 代码格式（不影响功能）
refactor: 重构
perf: 性能优化
test: 测试相关
chore: 构建/工具相关

示例:
feat(session): 添加会话卡片组件
fix(chat): 修复消息列表滚动问题
docs: 更新 API 文档
```

---

**文档变更记录**

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| v1.1 | 2026-01-29 | 添加 UI 设计规范章节，明确引用 prototype.html 作为 UI 设计规范来源；更新设计令牌和 TailwindCSS 配置与原型保持一致 | - |
| v1.0 | 2026-01-29 | 初始版本，完整前端架构设计（MVP 聚焦） | - |

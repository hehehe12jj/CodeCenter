import React, { useMemo } from 'react';
import { cn } from '@/utils/cn';
import type { Session } from '@/types/session';
import { StatusBadge } from '@/components/common/StatusBadge/StatusBadge';
import { formatRelativeTime } from '@/utils/formatters';
import {
  Play,
  Clock,
  CheckCircle2,
  AlertCircle,
  ExternalLink,
  MessageSquare,
  Archive,
  CheckCircle,
} from 'lucide-react';

interface SessionCardProps {
  session: Session;
  isSelected?: boolean;
  onSelect?: (session: Session) => void;
  onOpenTerminal?: (projectPath: string) => void;
  onOpenChat?: (sessionId: string) => void;
  onMarkCompleted?: (sessionId: string) => void;
  onArchive?: (sessionId: string) => void;
  className?: string;
}

/** 解析 title，提取文件夹名和 prompt */
function parseTitle(title: string): { folderName: string; prompt: string } {
  const separator = ' | ';
  const index = title.indexOf(separator);
  if (index === -1) {
    return { folderName: title, prompt: '' };
  }
  return {
    folderName: title.slice(0, index),
    prompt: title.slice(index + separator.length),
  };
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

/**
 * SessionCard 会话卡片组件
 *
 * 展示会话的基本信息、状态和快捷操作
 */
export const SessionCard: React.FC<SessionCardProps> = ({
  session,
  isSelected = false,
  onSelect,
  onOpenTerminal,
  onOpenChat,
  onMarkCompleted,
  onArchive,
  className,
}) => {
  const status = statusConfig[session.status];
  const StatusIcon = status.icon;

  const formattedTime = useMemo(() => {
    return formatRelativeTime(session.lastActiveAt);
  }, [session.lastActiveAt]);

  const handleCardClick = () => {
    onSelect?.(session);
  };

  const handleOpenTerminal = (e: React.MouseEvent) => {
    e.stopPropagation();
    onOpenTerminal?.(session.projectPath);
  };

  const handleOpenChat = (e: React.MouseEvent) => {
    e.stopPropagation();
    onOpenChat?.(session.id);
  };

  const handleMarkCompleted = (e: React.MouseEvent) => {
    e.stopPropagation();
    onMarkCompleted?.(session.id);
  };

  return (
    <div
      className={cn(
        'group relative overflow-hidden rounded-xl border border-border bg-bg-card/60',
        'backdrop-blur-xl transition-all duration-200',
        'hover:border-border-hover hover:bg-bg-tertiary/80',
        'cursor-pointer',
        isSelected && 'ring-2 ring-status-running/50 border-status-running',
        className
      )}
      onClick={handleCardClick}
    >
      {/* 状态指示灯 */}
      <div className="absolute left-0 top-0 bottom-0 w-1">
        <div
          className={cn(
            'absolute inset-y-0 left-0 w-full',
            status.bgColor,
            status.pulse && 'animate-pulse-slow'
          )}
        />
      </div>

      <div className="p-4 pl-5">
        {/* 头部 */}
        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 min-w-0">
            {/* 标题：文件夹名 | prompt */}
            <h3 className="font-medium text-text-primary truncate">
              {(() => {
                const { folderName, prompt } = parseTitle(session.title);
                return (
                  <>
                    <span className="text-sm">{folderName}</span>
                    {prompt && (
                      <>
                        <span className="mx-1 text-text-tertiary">|</span>
                        <span className="text-base font-normal">{prompt}</span>
                      </>
                    )}
                  </>
                );
              })()}
            </h3>
            <p className="text-sm text-text-secondary truncate mt-0.5">
              {session.projectName}
            </p>
          </div>

          {/* 状态徽章 */}
          <StatusBadge status={session.status} size="sm" />
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
              onClick={handleOpenChat}
              className="p-1.5 rounded-md hover:bg-white/10 transition-colors"
              title="继续对话"
            >
              <MessageSquare className="w-4 h-4" />
            </button>
            <button
              onClick={handleMarkCompleted}
              className="p-1.5 rounded-md hover:bg-white/10 transition-colors"
              title="标记完成"
            >
              <CheckCircle className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* 悬停效果 */}
      <div className="absolute inset-0 pointer-events-none ring-1 ring-inset ring-white/5 rounded-xl" />
    </div>
  );
};

// 引入依赖
import { formatRelativeTime } from '@/utils/formatters';

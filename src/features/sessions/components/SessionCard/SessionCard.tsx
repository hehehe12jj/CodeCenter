import React, { useMemo } from 'react';
import { cn } from '@/utils/cn';
import type { Session } from '@/types/session';
import { StatusBadge } from '@/components/common/StatusBadge/StatusBadge';
import { ProjectIcon } from '@/utils/project-icons';
import { formatRelativeTime } from '@/utils/formatters';
import { getDirectoryName } from '@/utils/path-utils';
import {
  Play,
  AlertCircle,
  MessageSquare,
  Clock,
  Loader2,
  CheckCircle,
} from 'lucide-react';

interface SessionCardProps {
  session: Session;
  isSelected?: boolean;
  onSelect?: (session: Session) => void;
  onOpenChat?: (sessionId: string) => void;
  className?: string;
}

const statusConfig: Record<string, {
  icon: React.ElementType;
  color: string;
  bgColor: string;
  borderColor: string;
  pulse: boolean;
  label: string;
  spin?: boolean;
}> = {
  running: {
    icon: Play,
    color: 'text-status-running',
    bgColor: 'bg-status-running/20',
    borderColor: 'border-status-running/30',
    pulse: true,
    label: '运行中',
  },
  waiting_input: {
    icon: Clock,
    color: 'text-status-waiting',
    bgColor: 'bg-status-waiting/20',
    borderColor: 'border-status-waiting/30',
    pulse: false,
    label: '等待输入',
  },
  blocked: {
    icon: AlertCircle,
    color: 'text-status-blocked',
    bgColor: 'bg-status-blocked/20',
    borderColor: 'border-status-blocked/30',
    pulse: true,
    label: '阻塞',
  },
  completed: {
    icon: CheckCircle,
    color: 'text-status-completed',
    bgColor: 'bg-status-completed/20',
    borderColor: 'border-status-completed/30',
    pulse: false,
    label: '已完成',
  },
  archived: {
    icon: CheckCircle,
    color: 'text-status-completed',
    bgColor: 'bg-status-completed/10',
    borderColor: 'border-status-completed/20',
    pulse: false,
    label: '已归档',
  },
  unknown: {
    icon: AlertCircle,
    color: 'text-gray-400',
    bgColor: 'bg-gray-400/10',
    borderColor: 'border-gray-400/20',
    pulse: false,
    label: '未知',
  },
  initializing: {
    icon: Loader2,
    color: 'text-status-initializing',
    bgColor: 'bg-status-initializing/20',
    borderColor: 'border-status-initializing/30',
    pulse: true,
    label: '初始化中',
    spin: true,
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
  onOpenChat,
  className,
}) => {
  const status = statusConfig[session.status];
  const StatusIcon = status.icon;

  const formattedTime = useMemo(() => {
    return formatRelativeTime(session.lastActiveAt);
  }, [session.lastActiveAt]);

  const simplifiedPath = useMemo(() => {
    // 只显示最后一级文件夹名
    return getDirectoryName(session.projectPath);
  }, [session.projectPath]);

  const handleCardClick = () => {
    onSelect?.(session);
  };

  const handleOpenChat = (e: React.MouseEvent) => {
    e.stopPropagation();
    onOpenChat?.(session.id);
  };

  return (
    <div
      className={cn(
        'group relative overflow-hidden rounded-xl border border-border bg-bg-card/60',
        'backdrop-blur-xl transition-all duration-200 ease-out',
        'hover:border-border-hover hover:bg-bg-tertiary/80',
        'hover:shadow-lg hover:shadow-black/20',
        'hover:-translate-y-0.5',
        'cursor-pointer',
        isSelected && 'ring-2 ring-status-running/50 border-status-running',
        className
      )}
      onClick={handleCardClick}
    >
      {/* 状态指示条（左侧） */}
      <div className="absolute left-0 top-0 bottom-0 w-1">
        <div
          className={cn(
            'absolute inset-y-0 left-0 w-full',
            status.bgColor,
            status.pulse && 'animate-pulse-slow'
          )}
        />
      </div>

      {/* 状态徽章（右上角） */}
      <div className="absolute top-3 right-3">
        <StatusBadge
          status={session.status}
          size="sm"
          className="shadow-sm"
        />
      </div>

      <div className="p-4 pl-5">
        {/* 头部：图标 + 标题 */}
        <div className="flex items-start gap-3 pr-20">
          {/* 项目类型图标 */}
          <ProjectIcon
            projectPath={session.projectPath}
            size="md"
          />

          {/* 标题区域 */}
          <div className="flex-1 min-w-0">
            {/* 主标题：项目名 | 第一条用户消息 */}
            <h3 className="font-semibold text-text-primary truncate text-base">
              {session.title}
            </h3>
            {/* 副标题：简化路径 */}
            <p className="text-xs text-text-tertiary truncate mt-0.5 font-mono">
              {simplifiedPath}
            </p>
          </div>
        </div>

        {/* 底部：时间 + 操作按钮 */}
        <div className="mt-3 flex items-center justify-between text-xs text-text-tertiary">
          <div className="flex items-center gap-2">
            <StatusIcon className={cn('w-3.5 h-3.5', status.color)} />
            <span>活跃于 {formattedTime}</span>
          </div>

          {/* 操作按钮：只保留继续对话 */}
          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
            <button
              onClick={handleOpenChat}
              className="p-1.5 rounded-md hover:bg-white/10 transition-colors"
              title="继续对话"
            >
              <MessageSquare className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* 悬停边框效果 */}
      <div className="absolute inset-0 pointer-events-none ring-1 ring-inset ring-white/5 rounded-xl group-hover:ring-white/10 transition-colors" />
    </div>
  );
};

export default SessionCard;

import React from 'react';
import { cn } from '@/utils/cn';
import type { SessionStatus } from '@/types/session';
import { Play, AlertCircle, CheckCircle, Clock, Loader2, type LucideIcon } from 'lucide-react';

interface StatusBadgeProps {
  /**
   * 会话状态
   */
  status: SessionStatus;
  /**
   * 是否显示脉冲动画
   * @default true
   */
  pulse?: boolean;
  /**
   * 尺寸
   * @default 'md'
   */
  size?: 'sm' | 'md' | 'lg';
  /**
   * 额外的 CSS 类名
   */
  className?: string;
  /**
   * 自定义图标
   */
  icon?: LucideIcon;
  /**
   * 显示文本
   */
  children?: React.ReactNode;
}

/**
 * 状态配置映射
 */
const statusConfig: Record<
  SessionStatus,
  {
    icon: LucideIcon;
    color: string;
    bgColor: string;
    borderColor: string;
    label: string;
    pulse: boolean;
    spin?: boolean;
  }
> = {
  running: {
    icon: Play,
    color: 'text-status-running',
    bgColor: 'bg-status-running/20',
    borderColor: 'border-status-running/30',
    label: '运行中',
    pulse: true,
  },
  waiting_input: {
    icon: Clock,
    color: 'text-status-waiting',
    bgColor: 'bg-status-waiting/20',
    borderColor: 'border-status-waiting/30',
    label: '等待输入',
    pulse: false,
  },
  blocked: {
    icon: AlertCircle,
    color: 'text-status-blocked',
    bgColor: 'bg-status-blocked/20',
    borderColor: 'border-status-blocked/30',
    label: '阻塞',
    pulse: true,
  },
  completed: {
    icon: CheckCircle,
    color: 'text-status-completed',
    bgColor: 'bg-status-completed/20',
    borderColor: 'border-status-completed/30',
    label: '已完成',
    pulse: false,
  },
  archived: {
    icon: CheckCircle,
    color: 'text-status-completed',
    bgColor: 'bg-status-completed/10',
    borderColor: 'border-status-completed/20',
    label: '已归档',
    pulse: false,
  },
  unknown: {
    icon: AlertCircle,
    color: 'text-gray-400',
    bgColor: 'bg-gray-400/10',
    borderColor: 'border-gray-400/20',
    label: '未知',
    pulse: false,
  },
  initializing: {
    icon: Loader2,
    color: 'text-status-running',
    bgColor: 'bg-status-running/20',
    borderColor: 'border-status-running/30',
    label: '运行中',
    pulse: true,
    spin: true,
  },
};

/**
 * StatusBadge 状态徽章组件
 *
 * @example
 * ```tsx
 * <StatusBadge status="running" />
 * <StatusBadge status="blocked" size="lg" pulse={false} />
 * <StatusBadge status="completed" children="自定义文本" />
 * ```
 */
export const StatusBadge: React.FC<StatusBadgeProps> = ({
  status,
  pulse,
  size = 'md',
  className,
  icon: CustomIcon,
  children,
}) => {
  const config = statusConfig[status];
  const Icon = CustomIcon || config.icon;
  const shouldPulse = pulse !== undefined ? pulse : config.pulse;
  const shouldSpin = config.spin || false;

  const sizeClasses = {
    sm: 'text-xs px-2 py-0.5 gap-1',
    md: 'text-xs px-2.5 py-1 gap-1.5',
    lg: 'text-sm px-3 py-1.5 gap-2',
  };

  const iconSizes = {
    sm: 'w-3 h-3',
    md: 'w-3.5 h-3.5',
    lg: 'w-4 h-4',
  };

  return (
    <span
      className={cn(
        'inline-flex items-center font-medium rounded-full border',
        config.bgColor,
        config.color,
        config.borderColor,
        sizeClasses[size],
        shouldPulse && 'animate-pulse-slow',
        className
      )}
    >
      <Icon className={cn(iconSizes[size], shouldSpin && 'animate-spin-slow')} />
      {children || config.label}
    </span>
  );
};

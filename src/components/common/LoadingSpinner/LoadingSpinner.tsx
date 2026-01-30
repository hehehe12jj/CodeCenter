import React from 'react';
import { cn } from '@/utils/cn';

export interface LoadingSpinnerProps {
  /**
   * 大小
   * @default 'md'
   */
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  /**
   * 颜色
   * @default 'currentColor'
   */
  color?: string;
  /**
   * 额外的 CSS 类名
   */
  className?: string;
}

/**
 * LoadingSpinner 加载动画组件
 *
 * @example
 * ```tsx
 * <LoadingSpinner size="md" />
 * <LoadingSpinner size="lg" color="#3b82f6" />
 * ```
 */
export const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({
  size = 'md',
  color = 'currentColor',
  className,
}) => {
  const sizeClasses = {
    xs: 'w-3 h-3',
    sm: 'w-4 h-4',
    md: 'w-5 h-5',
    lg: 'w-6 h-6',
    xl: 'w-8 h-8',
  };

  return (
    <svg
      className={cn('animate-spin', sizeClasses[size], className)}
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      style={{ color }}
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
      />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      />
    </svg>
  );
};

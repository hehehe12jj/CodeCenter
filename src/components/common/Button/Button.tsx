import React from 'react';
import { cn } from '@/utils/cn';
import { LoadingSpinner } from '../LoadingSpinner/LoadingSpinner';

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /**
   * 按钮变体
   * @default 'primary'
   */
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  /**
   * 按钮尺寸
   * @default 'md'
   */
  size?: 'sm' | 'md' | 'lg';
  /**
   * 是否加载中
   * @default false
   */
  loading?: boolean;
  /**
   * 是否禁用
   * @default false
   */
  disabled?: boolean;
  /**
   * 子元素
   */
  children: React.ReactNode;
}

/**
 * Button 基础按钮组件
 *
 * @example
 * ```tsx
 * <Button variant="primary" size="md">
 *   点击我
 * </Button>
 * <Button variant="ghost" size="sm" loading>
 *   加载中
 * </Button>
 * ```
 */
export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      loading = false,
      disabled = false,
      children,
      className,
      ...props
    },
    ref
  ) => {
    const baseStyles =
      'inline-flex items-center justify-center rounded-md font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-bg-primary disabled:opacity-50 disabled:cursor-not-allowed';

    const variants = {
      primary:
        'bg-status-running text-white hover:bg-status-running/90 focus:ring-status-running',
      secondary:
        'bg-bg-tertiary text-text-primary border border-border hover:border-border-hover hover:bg-bg-card focus:ring-text-secondary',
      ghost:
        'bg-transparent text-text-secondary hover:text-text-primary hover:bg-bg-glass focus:ring-text-secondary',
      danger:
        'bg-status-blocked text-white hover:bg-status-blocked/90 focus:ring-status-blocked',
    };

    const sizes = {
      sm: 'px-3 py-1.5 text-sm',
      md: 'px-4 py-2 text-sm',
      lg: 'px-6 py-3 text-base',
    };

    return (
      <button
        ref={ref}
        className={cn(baseStyles, variants[variant], sizes[size], className)}
        disabled={disabled || loading}
        {...props}
      >
        {loading ? (
          <>
            <LoadingSpinner className="mr-2 -ml-1 w-4 h-4" />
            加载中
          </>
        ) : (
          children
        )}
      </button>
    );
  }
);

Button.displayName = 'Button';

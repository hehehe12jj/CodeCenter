import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * 合并类名工具函数
 * 结合 clsx 和 tailwind-merge，支持条件类名和冲突解决
 *
 * @param inputs - 类名数组（支持字符串、对象、数组、布尔值等）
 * @returns 合并后的类名字符串
 *
 * @example
 * cn('flex', isActive && 'bg-blue-500', { 'text-white': true })
 * // => 'flex bg-blue-500 text-white'
 */
export function cn(...inputs: (string | undefined | null | false)[]) {
  return twMerge(clsx(inputs));
}

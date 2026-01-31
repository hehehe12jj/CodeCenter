/**
 * 格式化相对时间（如：刚刚、5分钟前、1小时前等）
 */
export function formatRelativeTime(date: string | Date): string {
  const now = new Date();
  const then = new Date(date);
  const diffInSeconds = Math.floor((now.getTime() - then.getTime()) / 1000);

  if (diffInSeconds < 60) {
    return '刚刚';
  }

  const diffInMinutes = Math.floor(diffInSeconds / 60);
  if (diffInMinutes < 60) {
    return `${diffInMinutes} 分钟前`;
  }

  const diffInHours = Math.floor(diffInMinutes / 60);
  if (diffInHours < 24) {
    return `${diffInHours} 小时前`;
  }

  const diffInDays = Math.floor(diffInHours / 24);
  if (diffInDays < 30) {
    return `${diffInDays} 天前`;
  }

  // 超过一个月，显示日期
  return then.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/**
 * 格式化日期时间
 */
export function formatDateTime(date: string | Date): string {
  const d = new Date(date);
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/**
 * 格式化时长（秒 -> 可读时间）
 */
export function formatDuration(seconds: number): string {
  if (seconds < 60) {
    return `${seconds} 秒`;
  }

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes} 分钟`;
  }

  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  if (remainingMinutes === 0) {
    return `${hours} 小时`;
  }

  return `${hours} 小时 ${remainingMinutes} 分钟`;
}

/**
 * 格式化消息摘要
 * - 移除 Claude Code 日志标签
 * - 智能识别场景并显示友好文案
 * - 处理空状态和截断
 */
export function formatMessageSummary(rawContent: string | undefined): string {
  // 空状态处理
  if (!rawContent || rawContent.trim() === '') {
    return '';
  }

  // 1. 移除 Claude Code 日志标签
  let clean = rawContent
    .replace(/<ide_selection>/g, '')
    .replace(/<\/ide_selection>/g, '')
    .replace(/<function_calls>/g, '')
    .replace(/<\/function_calls>/g, '')
    .replace(/<thinking>/g, '')
    .replace(/<\/thinking>/g, '')
    .replace(/<command>/g, '')
    .replace(/<\/command>/g, '')
    .replace(/<prompt>/g, '')
    .replace(/<\/prompt>/g, '')
    .trim();

  // 如果清理后为空，返回空字符串
  if (!clean) {
    return '';
  }

  // 2. 智能识别场景（返回 emoji + 描述）
  const lowerContent = clean.toLowerCase();

  // 用户选择代码
  if (lowerContent.includes('selected the lines') ||
      lowerContent.includes('selected lines') ||
      lowerContent.includes('用户选择了')) {
    return '👆 用户选择了代码片段';
  }

  // 工具函数调用
  if ((lowerContent.includes('function') && lowerContent.includes('call')) ||
      lowerContent.includes('调用工具') ||
      lowerContent.includes('executing tool')) {
    return '⚡ 正在调用工具函数';
  }

  // AI 思考中
  if (lowerContent.includes('thinking') ||
      lowerContent.includes('分析中') ||
      lowerContent.includes('让我思考一下')) {
    return '🤔 AI 正在思考...';
  }

  // 错误/阻塞
  if (lowerContent.includes('error') ||
      lowerContent.includes('错误') ||
      lowerContent.includes('exception') ||
      lowerContent.includes('failed') ||
      lowerContent.includes('失败')) {
    return '⚠️ 遇到错误，需要处理';
  }

  // 用户提问
  if (lowerContent.includes('?') ||
      lowerContent.includes('？') ||
      lowerContent.includes('如何') ||
      lowerContent.includes('怎么') ||
      lowerContent.includes('请问')) {
    return '❓ 用户提出问题';
  }

  // 代码相关
  if (lowerContent.includes('```') ||
      lowerContent.includes('code') ||
      lowerContent.includes('代码')) {
    return '💻 讨论代码实现';
  }

  // 3. 截断处理（最大 80 字符）
  const maxLength = 80;
  if (clean.length > maxLength) {
    return clean.slice(0, maxLength) + '...';
  }

  return clean;
}

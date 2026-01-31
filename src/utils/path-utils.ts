/**
 * 路径工具函数
 */

/**
 * 简化路径显示
 * 将完整路径转换为 ".../parent/projectName" 格式
 *
 * @example
 * simplifyPath('/Users/hejj/projects/my-project')
 * // 返回: '.../projects/my-project'
 *
 * simplifyPath('/home/user/code/project')
 * // 返回: '.../code/project'
 *
 * simplifyPath('/short/path')
 * // 返回: '/short/path' (路径太短，不做简化)
 */
export function simplifyPath(fullPath: string): string {
  if (!fullPath || typeof fullPath !== 'string') {
    return '';
  }

  // 统一使用 / 作为分隔符
  const normalizedPath = fullPath.replace(/\\/g, '/');
  const parts = normalizedPath.split('/').filter(Boolean);

  // 如果路径段数少于等于 2，直接返回
  if (parts.length <= 2) {
    return normalizedPath;
  }

  // 显示最后两段: .../parent/projectName
  const lastTwoParts = parts.slice(-2);
  return `.../${lastTwoParts.join('/')}`;
}

/**
 * 从完整路径中提取目录名
 *
 * @example
 * getDirectoryName('/Users/hejj/projects/my-project')
 * // 返回: 'my-project'
 */
export function getDirectoryName(fullPath: string): string {
  if (!fullPath || typeof fullPath !== 'string') {
    return '';
  }

  const normalizedPath = fullPath.replace(/\\/g, '/');
  const parts = normalizedPath.split('/').filter(Boolean);

  return parts[parts.length - 1] || '';
}

/**
 * 获取父目录名
 *
 * @example
 * getParentDirectoryName('/Users/hejj/projects/my-project')
 * // 返回: 'projects'
 */
export function getParentDirectoryName(fullPath: string): string {
  if (!fullPath || typeof fullPath !== 'string') {
    return '';
  }

  const normalizedPath = fullPath.replace(/\\/g, '/');
  const parts = normalizedPath.split('/').filter(Boolean);

  return parts[parts.length - 2] || '';
}

/**
 * 格式化项目标题
 * 根据 projectName 和 projectPath 生成友好的显示标题
 */
export function formatProjectTitle(
  projectName: string,
  projectPath: string
): {
  primary: string;    // 主标题 (projectName)
  secondary: string;  // 副标题 (简化路径)
} {
  return {
    primary: projectName || getDirectoryName(projectPath),
    secondary: simplifyPath(projectPath),
  };
}

import { QueryClient } from '@tanstack/react-query';

/**
 * TanStack Query Client 配置
 * 用于全局状态管理和数据获取
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // 30 秒内数据视为新鲜，不会重新获取
      staleTime: 30 * 1000,
      // 5 分钟后垃圾回收
      gcTime: 5 * 60 * 1000,
      // 失败时重试 1 次
      retry: 1,
      // 窗口聚焦时不自动刷新
      refetchOnWindowFocus: false,
    },
    mutations: {
      // 默认不重试 mutations
      retry: 0,
    },
  },
});

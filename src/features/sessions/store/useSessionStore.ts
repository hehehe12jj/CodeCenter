import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import type { Session } from '@/types/session';

interface SessionState {
  // 数据
  sessions: Session[];
  selectedSessionId: string | null;
  loading: boolean;
  error: string | null;

  // 轮询控制
  pollingInterval: number;
  pollingTimer: number | null;

  // 操作
  fetchSessions: () => Promise<void>;
  selectSession: (id: string | null) => void;
  markCompleted: (id: string) => Promise<void>;
  archiveSession: (id: string) => Promise<void>;

  // 轮询控制
  startPolling: () => void;
  stopPolling: () => void;
}

const DEFAULT_POLLING_INTERVAL = 10000; // 10秒

export const useSessionStore = create<SessionState>()(
  devtools(
    (set, get) => ({
      // 初始状态
      sessions: [],
      selectedSessionId: null,
      loading: false,
      error: null,
      pollingInterval: DEFAULT_POLLING_INTERVAL,
      pollingTimer: null,

      // 获取所有会话
      fetchSessions: async () => {
        // 防止并发请求
        const { loading } = get();
        if (loading) {
          console.log('[fetchSessions] 请求已在进行中，跳过');
          return;
        }

        set({ loading: true, error: null });
        try {
          console.log('[fetchSessions] 开始调用 get_all_sessions...');
          const rawData = await invoke<unknown>('get_all_sessions');
          console.log('[fetchSessions] 原始返回数据:', rawData);

          // 检查数据类型
          if (!Array.isArray(rawData)) {
            throw new Error(`返回数据不是数组: ${typeof rawData}`);
          }

          // 转换为 Session 数组
          const sessions = (rawData as unknown[]).map((item: unknown) => {
            return item as Session;
          });

          set({ sessions, loading: false });
          console.log('[fetchSessions] 会话加载成功:', sessions.length);
        } catch (error) {
          console.error('[fetchSessions] 错误:', error);
          set({ error: String(error), loading: false });
        }
      },

      // 选择会话
      selectSession: (id) => set({ selectedSessionId: id }),

      // 标记完成
      markCompleted: async (id) => {
        try {
          await invoke('mark_session_completed', { id });
          // 刷新会话列表
          await get().fetchSessions();
        } catch (error) {
          console.error('[markCompleted] 错误:', error);
          throw error;
        }
      },

      // 归档会话
      archiveSession: async (id) => {
        try {
          await invoke('archive_session', { id });
          // 刷新会话列表
          await get().fetchSessions();
        } catch (error) {
          console.error('[archiveSession] 错误:', error);
          throw error;
        }
      },

      // 启动轮询
      startPolling: () => {
        const { pollingInterval, fetchSessions, pollingTimer } = get();

        // 如果已有定时器，先停止
        if (pollingTimer !== null) {
          window.clearInterval(pollingTimer);
        }

        console.log(`[startPolling] 启动轮询，间隔 ${pollingInterval}ms`);

        // 立即执行一次
        fetchSessions();

        // 设置定时器
        const timer = window.setInterval(() => {
          console.log('[polling] 定时触发 fetchSessions');
          fetchSessions();
        }, pollingInterval);

        set({ pollingTimer: timer });
      },

      // 停止轮询
      stopPolling: () => {
        const { pollingTimer } = get();
        if (pollingTimer !== null) {
          window.clearInterval(pollingTimer);
          console.log('[stopPolling] 轮询已停止');
          set({ pollingTimer: null });
        }
      },
    }),
    { name: 'SessionStore' }
  )
);

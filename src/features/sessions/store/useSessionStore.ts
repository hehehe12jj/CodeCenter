import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { Session, SessionStatus } from '@/types/session';

interface SessionState {
  // 数据
  sessions: Session[];
  selectedSessionId: string | null;
  loading: boolean;
  error: string | null;

  // 操作
  fetchSessions: () => Promise<void>;
  selectSession: (id: string | null) => void;
  updateSessionStatus: (id: string, status: SessionStatus) => void;
  markCompleted: (id: string) => Promise<void>;
  archiveSession: (id: string) => Promise<void>;
  addSession: (session: Session) => void;

  // 事件监听
  initEventListeners: () => Promise<() => void>;
}

export const useSessionStore = create<SessionState>()(
  devtools(
    (set, get) => ({
      // 初始状态
      sessions: [],
      selectedSessionId: null,
      loading: false,
      error: null,

      // 获取所有会话
      fetchSessions: async () => {
        set({ loading: true, error: null });
        try {
          const sessions = await invoke<Session[]>('get_all_sessions');
          set({ sessions, loading: false });
        } catch (error) {
          set({ error: String(error), loading: false });
        }
      },

      // 选择会话
      selectSession: (id) => set({ selectedSessionId: id }),

      // 添加会话
      addSession: (session) =>
        set((state) => ({
          sessions: [session, ...state.sessions],
        })),

      // 更新会话状态
      updateSessionStatus: (id, status) =>
        set((state) => ({
          sessions: state.sessions.map((s) =>
            s.id === id ? { ...s, status } : s
          ),
        })),

      // 标记完成
      markCompleted: async (id) => {
        try {
          await invoke('mark_session_completed', { id });
          set((state) => ({
            sessions: state.sessions.map((s) =>
              s.id === id ? { ...s, status: 'completed' as SessionStatus } : s
            ),
          }));
        } catch (error) {
          get().fetchSessions();
          throw error;
        }
      },

      // 归档会话
      archiveSession: async (id) => {
        try {
          await invoke('archive_session', { id });
          set((state) => ({
            sessions: state.sessions.map((s) =>
              s.id === id ? { ...s, isArchived: true } : s
            ),
          }));
        } catch (error) {
          get().fetchSessions();
          throw error;
        }
      },

      // 初始化事件监听
      initEventListeners: async () => {
        const unlisteners: UnlistenFn[] = [];

        // 监听会话状态变更
        const statusUnlisten = await listen<{
          sessionId: string;
          newStatus: SessionStatus;
        }>('session:status-changed', (event) => {
          get().updateSessionStatus(
            event.payload.sessionId,
            event.payload.newStatus
          );
        });
        unlisteners.push(statusUnlisten);

        // 监听会话发现
        const discoveredUnlisten = await listen<Session>('session:discovered', (event) => {
          get().addSession(event.payload);
        });
        unlisteners.push(discoveredUnlisten);

        // 返回清理函数
        return () => {
          unlisteners.forEach((fn) => fn());
        };
      },
    }),
    { name: 'SessionStore' }
  )
);

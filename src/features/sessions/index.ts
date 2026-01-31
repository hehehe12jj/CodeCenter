import { invoke } from '@tauri-apps/api/core';
import { SessionCard } from './components/SessionCard/SessionCard';
import { useSessionStore } from '@/features/sessions/store/useSessionStore';

// 重新导出自定义 Hook
export { useSessionStore };

// 重新导出组件
export { SessionCard };

// 工具函数
export const sessionCommands = {
  getAllSessions: () => invoke('get_all_sessions'),
  markSessionCompleted: (id: string) => invoke('mark_session_completed', { id }),
  archiveSession: (id: string) => invoke('archive_session', { id }),
};

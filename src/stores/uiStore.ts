import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
}

interface UIState {
  // 侧边栏状态
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;

  // Toast 通知
  toasts: Toast[];
  addToast: (toast: Omit<Toast, 'id'>) => void;
  removeToast: (id: string) => void;

  // 主题
  theme: 'dark' | 'light';
  setTheme: (theme: 'dark' | 'light') => void;

  // 搜索状态
  searchQuery: string;
  searchActive: boolean;
  setSearchQuery: (query: string) => void;
  setSearchActive: (active: boolean) => void;
}

export const useUIStore = create<UIState>()(
  devtools(
    persist(
      (set, get) => ({
        // 侧边栏
        sidebarCollapsed: false,
        toggleSidebar: () =>
          set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

        // Toast
        toasts: [],
        addToast: (toast) => {
          const id = crypto.randomUUID();
          set((state) => ({
            toasts: [...state.toasts, { ...toast, id }],
          }));

          // 自动移除
          if (toast.duration !== 0) {
            setTimeout(() => {
              get().removeToast(id);
            }, toast.duration || 5000);
          }
        },
        removeToast: (id) =>
          set((state) => ({
            toasts: state.toasts.filter((t) => t.id !== id),
          })),

        // 主题
        theme: 'dark',
        setTheme: (theme) => set({ theme }),

        // 搜索
        searchQuery: '',
        searchActive: false,
        setSearchQuery: (query) => set({ searchQuery: query }),
        setSearchActive: (active) => set({ searchActive: active }),
      }),
      {
        name: 'ui-storage',
        partialize: (state) => ({
          sidebarCollapsed: state.sidebarCollapsed,
          theme: state.theme,
        }),
      }
    ),
    { name: 'UIStore' }
  )
);

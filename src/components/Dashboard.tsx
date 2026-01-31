import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useSessionStore } from "../features/sessions/store/useSessionStore";
import SessionCard from "../components/SessionCard";
import StatusBar from "./StatusBar";
import { RefreshCw } from "lucide-react";
import { cn } from "@/utils/cn";

export default function Dashboard() {
  // 使用 Zustand store 替代本地 state
  const {
    sessions,
    loading,
    error,
    fetchSessions,
  } = useSessionStore();

  const [lastUpdated, setLastUpdated] = useState<Date>(new Date());
  const [isCreating, setIsCreating] = useState(false);

  // 初始化：加载会话并启动轮询
  useEffect(() => {
    // 初始加载
    fetchSessions().then(() => {
      setLastUpdated(new Date());
    });

    // 启动轮询
    const { startPolling, stopPolling } = useSessionStore.getState();
    startPolling();

    // 组件卸载时停止轮询
    return () => {
      stopPolling();
    };
  }, [fetchSessions]);

  // 手动刷新
  const handleRefresh = async () => {
    await fetchSessions();
    setLastUpdated(new Date());
  };

  // 新建会话：选择项目目录并打开终端
  const handleCreateSession = async () => {
    try {
      setIsCreating(true);

      // 使用 Tauri 的 dialog 插件选择目录
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择项目目录",
      });

      if (selected && typeof selected === "string") {
        // 打开系统终端
        await invoke("open_terminal", { projectPath: selected });

        // 改进：多次重试刷新，确保获取到新会话
        let retries = 3;
        for (let i = 0; i < retries; i++) {
          await new Promise(resolve => setTimeout(resolve, 2000));
          await handleRefresh();

          // 检查是否获取到该项目的会话
          const currentSessions = useSessionStore.getState().sessions;
          const hasNewSession = currentSessions.some(
            s => s.projectPath === selected && !s.isArchived
          );
          if (hasNewSession) {
            console.log('[handleCreateSession] 成功获取新会话');
            break;
          }
          console.log(`[handleCreateSession] 第 ${i + 1} 次重试，未发现新会话`);
        }
      }
    } catch (error) {
      console.error("Failed to create session:", error);
      alert(`创建会话失败: ${error}`);
    } finally {
      setIsCreating(false);
    }
  };

  // 过滤会话：展示 Running、WaitingInput、Blocked、Initializing 状态
  // Initializing 也显示为运行中（表示会话已启动，正在初始化）
  const visibleSessions = sessions.filter(
    (s) =>
      s.status === 'running' ||
      s.status === 'waiting_input' ||
      s.status === 'blocked' ||
      s.status === 'initializing'
  );

  // 统计数据
  const runningCount = visibleSessions.filter((s) => s.status === 'running').length;
  const waitingInputCount = visibleSessions.filter((s) => s.status === 'waiting_input').length;
  const blockedCount = visibleSessions.filter((s) => s.status === 'blocked').length;
  const totalCount = visibleSessions.length;

  return (
    <div className="min-h-screen p-6">
      {/* Header */}
      <header className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-4">
          <img src="/trans_bg.png" alt="CodeCenter" className="w-12 h-12 rounded-2xl" />
          <h1 className="text-xl font-semibold">CodeCenter</h1>
        </div>
        <div className="flex items-center gap-3">
          <button
            className="glass-button px-4 py-2 text-sm text-white disabled:opacity50 disabled:cursor-not-allowed"
            onClick={handleCreateSession}
            disabled={isCreating}
          >
            {isCreating ? '创建中...' : '+ 新建'}
          </button>
          <button
            className="glass-button px-4 py-2 text-white disabled:opacity-50 flex items-center justify-center"
            onClick={handleRefresh}
            disabled={loading}
          >
            <RefreshCw className={cn("w-4 h-4", loading && "animate-spin")} />
          </button>
        </div>
      </header>

      {/* Error Message */}
      {error && (
        <div className="mb-4 p-4 bg-red-500/20 border border-red-500/30 rounded-lg text-red-200">
          <p className="font-medium">加载失败</p>
          <p className="text-sm opacity-80">{error}</p>
        </div>
      )}

      {/* Status Bar */}
      <StatusBar
        runningCount={runningCount}
        waitingInputCount={waitingInputCount}
        blockedCount={blockedCount}
        totalCount={totalCount}
        lastUpdated={lastUpdated}
      />

      {/* Session Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4 mt-6">
        {visibleSessions.length === 0 ? (
          <div className="glass-card p-8 col-span-full text-center text-white/60">
            <p>暂无活跃会话</p>
            <p className="text-sm mt-2">点击右上角 "+ 新建" 开始一个会话</p>
          </div>
        ) : (
          visibleSessions.map((session) => (
            <SessionCard
              key={session.id}
              session={session}
            />
          ))
        )}
      </div>
    </div>
  );
}

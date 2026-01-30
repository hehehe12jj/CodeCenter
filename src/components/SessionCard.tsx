import { Session } from "../types/session";

interface SessionCardProps {
  session: Session;
  onRefresh: () => void;
}

export default function SessionCard({ session, onRefresh: _onRefresh }: SessionCardProps) {
  const getStatusClass = (status: string) => {
    switch (status) {
      case "running":
        return "status-dot-running";
      case "waiting_input":
        return "status-dot-waiting";
      case "completed":
        return "status-dot-completed";
      case "blocked":
        return "status-dot-blocked";
      default:
        return "";
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case "running":
        return "运行中";
      case "waiting_input":
        return "等待输入";
      case "completed":
        return "已完成";
      case "blocked":
        return "执行阻塞";
      default:
        return "未知";
    }
  };

  const formatTime = (isoString: string) => {
    const date = new Date(isoString);
    return date.toLocaleTimeString("zh-CN", {
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <div className="glass-card p-5 hover:bg-white/5 transition-all duration-200 cursor-pointer group">
      <div className="flex items-start justify-between mb-3">
        <h3 className="font-medium text-white/90 truncate pr-4">
          {session.title}
        </h3>
        <div className="flex items-center gap-2 shrink-0">
          <span className={`status-dot ${getStatusClass(session.status)}`}></span>
          <span className="text-xs text-white/60">{getStatusText(session.status)}</span>
        </div>
      </div>

      <div className="text-sm text-white/40 mb-2">{session.projectName}</div>

      <div className="text-xs text-white/30 mb-4">
        开始: {formatTime(session.createdAt)} · 最后活跃: {formatTime(session.lastActiveAt)}
      </div>

      <div className="text-sm text-white/50 mb-4 truncate">
        {session.summary || "暂无消息摘要"}
      </div>

      <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
        <button className="glass-button px-3 py-1.5 text-xs text-white">
          继续对话
        </button>
        <button className="glass-button px-3 py-1.5 text-xs text-white">
          打开终端
        </button>
        <button className="glass-button px-3 py-1.5 text-xs text-white">
          标记完成
        </button>
      </div>
    </div>
  );
}

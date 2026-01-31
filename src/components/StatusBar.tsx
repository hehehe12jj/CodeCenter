interface StatusBarProps {
  runningCount: number;
  waitingInputCount: number;
  blockedCount: number;
  totalCount: number;
  lastUpdated: Date;
}

export default function StatusBar({
  runningCount,
  waitingInputCount,
  blockedCount,
  totalCount,
  lastUpdated,
}: StatusBarProps) {
  const formatTime = (date: Date) => {
    return date.toLocaleTimeString("zh-CN", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  return (
    <div className="glass-card p-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-8">
          <div className="flex items-center gap-2">
            <span className="status-dot status-dot-running"></span>
            <span className="text-2xl font-semibold">{runningCount}</span>
            <span className="text-white/60 text-sm">运行中</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="status-dot status-dot-waiting"></span>
            <span className="text-2xl font-semibold">{waitingInputCount}</span>
            <span className="text-white/60 text-sm">等待输入</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="status-dot status-dot-blocked"></span>
            <span className="text-2xl font-semibold">{blockedCount}</span>
            <span className="text-white/60 text-sm">阻塞</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-2xl font-semibold">{totalCount}</span>
            <span className="text-white/60 text-sm">总会话数</span>
          </div>
        </div>
        <div className="text-white/40 text-sm">
          最后更新: {formatTime(lastUpdated)}
        </div>
      </div>
    </div>
  );
}

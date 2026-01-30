interface StatusBarProps {
  runningCount: number;
  waitingCount: number;
  completedCount: number;
  lastUpdated: Date;
}

export default function StatusBar({
  runningCount,
  waitingCount,
  completedCount,
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
            <span className="text-2xl font-semibold">{waitingCount}</span>
            <span className="text-white/60 text-sm">等待输入</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="status-dot status-dot-completed"></span>
            <span className="text-2xl font-semibold">{completedCount}</span>
            <span className="text-white/60 text-sm">已完成</span>
          </div>
        </div>
        <div className="text-white/40 text-sm">
          最后更新: {formatTime(lastUpdated)}
        </div>
      </div>
    </div>
  );
}

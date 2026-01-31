import { useMemo } from 'react';
import { Session } from '../types/session';
import { formatRelativeTime } from '../utils/formatters';
import { getDirectoryName } from '../utils/path-utils';
import { MessageSquare } from 'lucide-react';

interface SessionCardProps {
  session: Session;
  isSelected?: boolean;
  onSelect?: (session: Session) => void;
  onOpenChat?: (sessionId: string) => void;
  className?: string;
}

const statusConfig = {
  running: {
    color: 'bg-status-running',
    label: '运行中',
  },
  waiting_input: {
    color: 'bg-status-waiting',
    label: '等待输入',
  },
  completed: {
    color: 'bg-status-completed',
    label: '已完成',
  },
  blocked: {
    color: 'bg-status-blocked',
    label: '阻塞',
  },
};

export default function SessionCard({
  session,
  isSelected = false,
  onSelect,
  onOpenChat,
  className,
}: SessionCardProps) {
  const status = statusConfig[session.status as keyof typeof statusConfig] || statusConfig.running;

  const formattedTime = useMemo(() => {
    return formatRelativeTime(session.lastActiveAt);
  }, [session.lastActiveAt]);

  const simplifiedPath = useMemo(() => {
    return getDirectoryName(session.projectPath);
  }, [session.projectPath]);

  const handleCardClick = () => {
    onSelect?.(session);
  };

  const handleOpenChat = (e: React.MouseEvent) => {
    e.stopPropagation();
    onOpenChat?.(session.id);
  };

  return (
    <div
      className={`
        glass-card p-4 hover:bg-white/5 transition-all duration-200 cursor-pointer group
        ${isSelected ? 'ring-2 ring-status-running/50 border-status-running' : ''}
        ${className || ''}
      `}
      onClick={handleCardClick}
    >
      {/* 状态指示条（左侧） */}
      <div className={`absolute left-0 top-0 bottom-0 w-1 ${status.color}`} />

      {/* 右上角状态 */}
      <div className="absolute top-3 right-3 flex items-center gap-1.5">
        <span className={`w-1.5 h-1.5 rounded-full ${status.color}`} />
        <span className="text-xs text-white/60">{status.label}</span>
      </div>

      <div className="pl-3">
        {/* 标题：项目名 | 第一条用户消息 */}
        <h3 className="font-medium text-white/90 truncate pr-8 mb-1">
          {session.title}
        </h3>

        {/* 路径 */}
        <div className="text-xs text-white/40 font-mono truncate mb-3">
          {simplifiedPath}
        </div>

        {/* 底部：时间 + 操作按钮 */}
        <div className="flex items-center justify-between">
          <div className="text-xs text-white/30">
            活跃于 {formattedTime}
          </div>

          {/* 只保留继续对话按钮 */}
          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
            <button
              onClick={handleOpenChat}
              className="glass-button px-3 py-1.5 text-xs text-white flex items-center gap-1.5"
            >
              <MessageSquare className="w-3.5 h-3.5" />
              继续对话
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

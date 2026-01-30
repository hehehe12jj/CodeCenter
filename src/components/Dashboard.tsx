import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Session } from "../types/session";
import SessionCard from "./SessionCard";
import StatusBar from "./StatusBar";

export default function Dashboard() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [lastUpdated, setLastUpdated] = useState<Date>(new Date());

  useEffect(() => {
    loadSessions();
  }, []);

  const loadSessions = async () => {
    try {
      const data = await invoke<Session[]>("get_all_sessions");
      setSessions(data);
      setLastUpdated(new Date());
    } catch (error) {
      console.error("Failed to load sessions:", error);
    }
  };

  const runningCount = sessions.filter((s) => s.status === "running").length;
  const waitingCount = sessions.filter((s) => s.status === "waiting_input").length;
  const completedCount = sessions.filter((s) => s.status === "completed").length;

  return (
    <div className="min-h-screen p-6">
      {/* Header */}
      <header className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-3">
          <span className="text-2xl">⟡</span>
          <h1 className="text-xl font-semibold">CodeAgent Dashboard</h1>
        </div>
        <div className="flex items-center gap-3">
          <button
            className="glass-button px-4 py-2 text-sm text-white"
            onClick={() => {}}
          >
            + 新建
          </button>
          <button
            className="glass-button px-4 py-2 text-sm text-white"
            onClick={loadSessions}
          >
            ↻
          </button>
        </div>
      </header>

      {/* Status Bar */}
      <StatusBar
        runningCount={runningCount}
        waitingCount={waitingCount}
        completedCount={completedCount}
        lastUpdated={lastUpdated}
      />

      {/* Session Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4 mt-6">
        {sessions.length === 0 ? (
          <div className="glass-card p-8 col-span-full text-center text-white/60">
            <p>暂无活跃会话</p>
            <p className="text-sm mt-2">点击右上角 "+ 新建" 开始一个会话</p>
          </div>
        ) : (
          sessions.map((session) => (
            <SessionCard
              key={session.id}
              session={session}
              onRefresh={loadSessions}
            />
          ))
        )}
      </div>
    </div>
  );
}

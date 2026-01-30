export type SessionStatus =
  | "running"
  | "waiting_input"
  | "completed"
  | "blocked";

export interface Session {
  id: string;
  title: string;
  projectName: string;
  projectPath: string;
  agentType: string;
  status: SessionStatus;
  createdAt: string;
  lastActiveAt: string;
  summary?: string;
  isArchived: boolean;
}

export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: string;
  metadata?: {
    hasCode: boolean;
    tokenCount?: number;
  };
}

export interface SessionDetail extends Session {
  messages: Message[];
  processInfo?: {
    pid: number;
    startTime: string;
    commandLine: string;
  };
  stats: {
    messageCount: number;
    totalTokens?: number;
    durationSecs: number;
  };
}

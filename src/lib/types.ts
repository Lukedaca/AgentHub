export interface Agent {
  id: string;
  name: string;
  shortName: string;
  command: string;
  args: string[];
  status: 'offline' | 'running' | 'error' | 'starting' | 'unavailable';
  color: string;
  version?: string;
  path?: string;
}

export interface Message {
  id: string;
  role: 'user' | 'agent' | 'system';
  text: string;
  timestamp: number;
  tokens?: number;
  files?: number;
}

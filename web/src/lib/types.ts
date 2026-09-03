export interface ParsedEvent {
  timestamp: string;
  hostname: string | null;
  service: string | null;
  severity: string | null;
  message: string;
  raw_event: string;
}

export interface Session {
  id: string;
  name: string;
  type: 'live' | 'static';
  events: ParsedEvent[];
  searchQuery: string;
  selectedSeverity: string;
  timeRange: [string, string] | null;
  isPaused: boolean;
  queuedEvents: ParsedEvent[];
}

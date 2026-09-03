export interface ParsedEvent {
  timestamp: string;
  hostname: string | null;
  service: string | null;
  severity: string | null;
  message: string;
  raw_event: string;
}

// Helios log parsing logic — Interacts with the Rust backend API with client-side fallback

export interface HeliosProcess {
  pid: number | null
  name: string | null
  thread_id?: number | null
  thread_name?: string | null
}

export interface HeliosNetwork {
  source_ip?: string | null
  source_port?: number | null
  destination_ip?: string | null
  destination_port?: number | null
  protocol?: string | null
}

export interface HeliosEvent {
  timestamp: string
  hostname: string | null
  service: string | null
  severity: string | null
  message: string
  raw_event: string
  process: HeliosProcess | null
  network: HeliosNetwork | null
  metadata: Record<string, any>
  attributes: Record<string, any>
  trace?: {
    trace_id: string
    span_id?: string | null
  } | null
}

export type DetectedFormat = "syslog" | "json" | "apache" | "nginx" | string | null

// RFC 5424 severity names (index = severity 0-7)
const SEVERITY_NAMES = [
  "EMERGENCY",
  "ALERT",
  "CRITICAL",
  "ERROR",
  "WARNING",
  "NOTICE",
  "INFORMATIONAL",
  "DEBUG",
] as const

function parsePri(input: string): { facility: number; severity: number } | null {
  if (!input.startsWith("<")) return null
  const closeIdx = input.indexOf(">")
  if (closeIdx < 2) return null
  const priStr = input.slice(1, closeIdx)
  const pri = parseInt(priStr, 10)
  if (isNaN(pri) || pri < 0 || pri > 191) return null
  return { facility: Math.floor(pri / 8), severity: pri % 8 }
}

function parseSyslogTimestamp(ts: string): string | null {
  if (/^\d{4}-\d{2}-\d{2}T/.test(ts)) {
    return ts
  }
  return new Date().toISOString()
}

/** Synchronous local format detection (used directly or as fallback) */
export function detectFormat(input: string): DetectedFormat {
  const trimmed = input.trim()
  const priResult = parsePri(trimmed)
  if (priResult !== null) {
    return "syslog"
  }
  // Check BSD syslog timestamp pattern: Month Day HH:MM:SS
  const bsdMatch = /^(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}/.test(trimmed)
  if (bsdMatch) {
    return "syslog"
  }
  try {
    JSON.parse(trimmed)
    return "json"
  } catch {
    return null
  }
}

/** Synchronous local syslog parsing */
export function parseSyslog(raw: string): HeliosEvent {
  const trimmed = raw.trim()
  let rest = trimmed
  let severityName: string | null = null

  const priResult = parsePri(rest)
  if (priResult) {
    severityName = SEVERITY_NAMES[priResult.severity]
    rest = rest.slice(rest.indexOf(">") + 1)
  }

  let timestamp = new Date().toISOString()
  let hostname: string | null = null
  let service: string | null = null
  let process: HeliosProcess | null = null
  let message = ""

  const rfc5424Match = rest.match(
    /^(\d)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(.*)/s
  )
  if (rfc5424Match) {
    const [, , ts, host, app, pid, , , msg] = rfc5424Match
    timestamp = parseSyslogTimestamp(ts) ?? timestamp
    hostname = host === "-" ? null : host
    service = app === "-" ? null : app
    const pidNum = pid === "-" ? null : parseInt(pid, 10)
    if (!isNaN(pidNum as number) && pidNum !== null) {
      process = { pid: pidNum, name: null }
    }
    message = msg ?? ""
  } else {
    const rfc3164Match = rest.match(
      /^(\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(\S+)\s+(.*)/s
    )
    if (rfc3164Match) {
      const [, ts, host, remainder] = rfc3164Match
      timestamp = parseSyslogTimestamp(ts) ?? timestamp
      hostname = host

      const tagMatch = remainder.match(/^(\S+?)(?:\[(\d+)\])?\s*:\s*(.*)/s)
      if (tagMatch) {
        const [, appname, pid, msg] = tagMatch
        service = appname || null
        message = msg ?? ""
        if (pid) {
          process = {
            pid: parseInt(pid, 10),
            name: null,
          }
        }
      } else {
        message = remainder
      }
    } else {
      message = rest
    }
  }

  return {
    timestamp,
    hostname,
    service,
    severity: severityName,
    message,
    raw_event: raw,
    process,
    network: null,
    metadata: {},
    attributes: {},
  }
}

/** Synchronous local JSON parsing */
export function parseJson(raw: string): HeliosEvent {
  const obj = JSON.parse(raw.trim())
  const isObj = typeof obj === "object" && obj !== null && !Array.isArray(obj)

  const message = isObj
    ? (typeof obj.message === "string" ? obj.message : String(obj.message ?? ""))
    : ""

  const severity = isObj
    ? (obj.level !== undefined ? String(obj.level) : obj.severity !== undefined ? String(obj.severity) : null)
    : null

  return {
    timestamp: new Date().toISOString(),
    hostname: isObj && obj.host ? String(obj.host) : null,
    service: isObj && obj.service ? String(obj.service) : null,
    severity,
    message,
    raw_event: raw,
    process: null,
    network: null,
    metadata: {},
    attributes: isObj ? obj : {},
  }
}

/* -------------------------------------------------------------------------- */
/* Async Backend API Connectors (Rust Engine)                                  */
/* -------------------------------------------------------------------------- */

const API_BASE = "/api/v1"

/** Check if the Rust Helios backend server is active */
export async function checkBackendHealth(): Promise<{ online: boolean; version?: string }> {
  try {
    const res = await fetch(`${API_BASE}/health`, { signal: AbortSignal.timeout(1500) })
    if (res.ok) {
      const data = await res.json()
      return { online: true, version: data.version }
    }
  } catch {
    // backend offline
  }
  return { online: false }
}

/** Detect format using Rust backend or local fallback */
export async function detectFormatWithBackend(raw: string): Promise<{ format: DetectedFormat; fromBackend: boolean }> {
  try {
    const res = await fetch(`${API_BASE}/detect`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ log: raw }),
      signal: AbortSignal.timeout(2000),
    })
    if (res.ok) {
      const data = await res.json()
      return { format: data.format, fromBackend: true }
    }
  } catch {
    // Fall back to client-side detection
  }
  return { format: detectFormat(raw), fromBackend: false }
}

/** Parse and normalize log line using Rust backend or local fallback */
export async function normalizeWithBackend(
  raw: string,
  format?: string | null
): Promise<{ format: string; event: HeliosEvent; fromBackend: boolean }> {
  try {
    const res = await fetch(`${API_BASE}/normalize`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ log: raw, format: format ?? undefined }),
      signal: AbortSignal.timeout(2500),
    })
    if (res.ok) {
      const data = await res.json()
      return { format: data.format, event: data.event, fromBackend: true }
    }
  } catch {
    // Fall back to client-side normalization
  }

  const detected = format || detectFormat(raw)
  if (detected === "syslog") {
    return { format: "syslog", event: parseSyslog(raw), fromBackend: false }
  }
  if (detected === "json") {
    return { format: "json", event: parseJson(raw), fromBackend: false }
  }
  throw new Error("Unable to parse log format")
}

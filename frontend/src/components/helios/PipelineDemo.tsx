import type React from "react"
import { useState, useRef, useCallback, useEffect } from "react"
import {
  detectFormatWithBackend,
  normalizeWithBackend,
  checkBackendHealth,
  type HeliosEvent,
  type DetectedFormat,
} from "@/lib/helios-parser"

type Stage = "idle" | "detecting" | "detected" | "parsing" | "parsed" | "normalizing" | "done" | "error"

const SYSLOG_SAMPLE = `<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8`
const JSON_SAMPLE = `{"level": "error", "message": "connection refused", "host": "db-01"}`

const STAGE_LABELS: Record<string, string> = {
  detecting: "Detect Format",
  detected: "Detect Format",
  parsing: "Parse",
  parsed: "Parse",
  normalizing: "Normalize",
  done: "Normalize",
}

function useReducedMotion() {
  const [reduced, setReduced] = useState(false)
  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)")
    setReduced(mq.matches)
    const handler = () => setReduced(mq.matches)
    mq.addEventListener("change", handler)
    return () => mq.removeEventListener("change", handler)
  }, [])
  return reduced
}

export function PipelineDemo() {
  const [activeTab, setActiveTab] = useState<"syslog" | "json">("syslog")
  const [inputValue, setInputValue] = useState(SYSLOG_SAMPLE)
  const [stage, setStage] = useState<Stage>("idle")
  const [detectedFormat, setDetectedFormat] = useState<DetectedFormat>(null)
  const [outputEvent, setOutputEvent] = useState<HeliosEvent | null>(null)
  const [errorMsg, setErrorMsg] = useState<string | null>(null)
  const [backendOnline, setBackendOnline] = useState<boolean>(false)
  const [usedBackend, setUsedBackend] = useState<boolean>(false)
  const reducedMotion = useReducedMotion()
  const runningRef = useRef(false)

  const isRunning = stage !== "idle" && stage !== "done" && stage !== "error"

  // Check Rust backend API connection status
  useEffect(() => {
    let mounted = true
    async function checkHealth() {
      const { online } = await checkBackendHealth()
      if (mounted) setBackendOnline(online)
    }
    checkHealth()
    const interval = setInterval(checkHealth, 5000)
    return () => {
      mounted = false
      clearInterval(interval)
    }
  }, [])

  function switchTab(tab: "syslog" | "json") {
    setActiveTab(tab)
    setInputValue(tab === "syslog" ? SYSLOG_SAMPLE : JSON_SAMPLE)
    setStage("idle")
    setDetectedFormat(null)
    setOutputEvent(null)
    setErrorMsg(null)
    setUsedBackend(false)
  }

  const delay = useCallback((ms: number) => {
    if (reducedMotion) return Promise.resolve()
    return new Promise<void>((res) => setTimeout(res, ms))
  }, [reducedMotion])

  async function runPipeline() {
    if (isRunning) return
    runningRef.current = true
    setStage("idle")
    setDetectedFormat(null)
    setOutputEvent(null)
    setErrorMsg(null)
    setUsedBackend(false)

    await delay(50)

    // Stage 1: detect format
    setStage("detecting")
    await delay(400)

    try {
      const { format: fmt, fromBackend } = await detectFormatWithBackend(inputValue)
      setDetectedFormat(fmt)
      if (fromBackend) setUsedBackend(true)

      if (!fmt) {
        setStage("error")
        setErrorMsg("Format not recognized — input does not match supported syslog or JSON patterns.")
        runningRef.current = false
        return
      }

      setStage("detected")
      await delay(350)

      // Stage 2 & 3: Parse and Normalize
      setStage("parsing")
      await delay(380)

      setStage("normalizing")
      await delay(400)

      const result = await normalizeWithBackend(inputValue, fmt)
      if (result.fromBackend) setUsedBackend(true)

      setOutputEvent(result.event)
      setStage("done")
    } catch (e: any) {
      setStage("error")
      setErrorMsg(e?.message || "Failed to process log line through the Helios pipeline.")
    } finally {
      runningRef.current = false
    }
  }

  const stageIndex = ["detecting", "detected", "parsing", "parsed", "normalizing", "done"]
  const currentStageIdx = stageIndex.indexOf(stage)

  return (
    <section
      id="pipeline-demo"
      className="px-6 py-24"
      style={{ background: "#000000" }}
    >
      <div className="mx-auto max-w-5xl">
        {/* Header */}
        <div className="mb-12 text-center">
          <div className="mb-3 flex items-center justify-center gap-2">
            <p className="font-mono text-xs uppercase tracking-widest" style={{ color: "#C8942E" }}>
              Live Demo
            </p>
            <span
              className="inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 font-mono text-[11px] font-medium"
              style={{
                background: backendOnline ? "#0E2A18" : "#211A11",
                color: backendOnline ? "#4ADE80" : "#A9A39A",
                border: `1px solid ${backendOnline ? "#22C55E44" : "#30271A"}`,
              }}
            >
              <span
                className="size-1.5 rounded-full"
                style={{
                  background: backendOnline ? "#22C55E" : "#6F6A62",
                  boxShadow: backendOnline ? "0 0 6px #22C55E" : "none",
                }}
              />
              {backendOnline ? "Rust API Connected (:8080)" : "Client Fallback Mode"}
            </span>
          </div>
          <h2
            className="mb-4 text-3xl font-bold tracking-tight md:text-4xl"
            style={{ color: "#F5F1E8", letterSpacing: "-0.02em" }}
          >
            Interactive Pipeline
          </h2>
          <p className="mx-auto max-w-xl text-sm leading-relaxed" style={{ color: "#A9A39A" }}>
            Paste any log line below and watch Helios detect, parse, and normalize it in real time through the Rust core engine.
          </p>
        </div>

        <div
          className="rounded-xl border"
          style={{ borderColor: "#30271A", background: "#0D0A06" }}
        >
          {/* Tab bar */}
          <div
            className="flex items-center justify-between border-b px-2"
            style={{ borderColor: "#211A11" }}
            role="tablist"
            aria-label="Log format selector"
          >
            <div className="flex items-center">
              {(["syslog", "json"] as const).map((tab) => (
                <button
                  key={tab}
                  role="tab"
                  aria-selected={activeTab === tab}
                  aria-controls={`panel-${tab}`}
                  id={`tab-${tab}`}
                  onClick={() => switchTab(tab)}
                  className="relative px-5 py-3 font-mono text-xs font-medium uppercase tracking-widest transition-colors duration-150 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
                  style={{
                    color: activeTab === tab ? "#C8942E" : "#6F6A62",
                    borderBottom: activeTab === tab ? "2px solid #C8942E" : "2px solid transparent",
                    background: "transparent",
                  }}
                >
                  {tab === "syslog" ? "Syslog" : "JSON"}
                </button>
              ))}
            </div>
            {usedBackend && (
              <span className="mr-3 font-mono text-[11px]" style={{ color: "#C8942E" }}>
                ⚡ Processed by Rust Backend
              </span>
            )}
          </div>

          <div className="grid gap-0 md:grid-cols-2">
            {/* Input panel */}
            <div
              className="border-b p-5 md:border-b-0 md:border-r"
              style={{ borderColor: "#211A11" }}
              id={`panel-${activeTab}`}
              role="tabpanel"
              aria-labelledby={`tab-${activeTab}`}
            >
              <p className="mb-2 font-mono text-xs uppercase tracking-widest" style={{ color: "#6F6A62" }}>
                Input
              </p>
              <textarea
                className="w-full resize-none rounded-md border p-3 font-mono text-xs leading-relaxed focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 disabled:opacity-50"
                style={{
                  background: "#171008",
                  borderColor: "#30271A",
                  color: "#F5F1E8",
                  minHeight: "120px",
                  fontFamily: "var(--font-mono)",
                }}
                value={inputValue}
                onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setInputValue(e.target.value)}
                disabled={isRunning}
                aria-label="Log input"
                spellCheck={false}
              />
              <button
                onClick={runPipeline}
                disabled={isRunning || !inputValue.trim()}
                className="mt-3 flex items-center gap-2 rounded-md px-4 py-2 text-sm font-semibold transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
                style={{
                  background: isRunning ? "#30271A" : "#C8942E",
                  color: isRunning ? "#6F6A62" : "#000000",
                }}
                onMouseEnter={(e: React.MouseEvent<HTMLButtonElement>) => {
                  if (!isRunning) e.currentTarget.style.background = "#E7B84B"
                }}
                onMouseLeave={(e: React.MouseEvent<HTMLButtonElement>) => {
                  if (!isRunning) e.currentTarget.style.background = "#C8942E"
                }}
              >
                {isRunning ? (
                  <>
                    <Spinner />
                    Running Pipeline…
                  </>
                ) : (
                  <>
                    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
                      <polygon points="3,2 12,7 3,12" fill="currentColor" />
                    </svg>
                    Run Pipeline
                  </>
                )}
              </button>
            </div>

            {/* Pipeline stages + output panel */}
            <div className="p-5">
              <p className="mb-3 font-mono text-xs uppercase tracking-widest" style={{ color: "#6F6A62" }}>
                Pipeline Stages
              </p>

              <div className="mb-4 flex flex-col gap-2">
                {[
                  { key: "detect", label: "Detect Format", stageStart: 0, stageEnd: 1 },
                  { key: "parse", label: "Parse", stageStart: 2, stageEnd: 3 },
                  { key: "normalize", label: "Normalize", stageStart: 4, stageEnd: 5 },
                ].map(({ key, label, stageStart, stageEnd }) => {
                  const isActive = currentStageIdx >= stageStart && currentStageIdx <= stageEnd && stage !== "error"
                  const isComplete = currentStageIdx > stageEnd && stage !== "error"
                  const isLoading = (
                    (key === "detect" && stage === "detecting") ||
                    (key === "parse" && stage === "parsing") ||
                    (key === "normalize" && stage === "normalizing")
                  ) && !reducedMotion

                  return (
                    <div
                      key={key}
                      className="flex items-center gap-3 rounded-md border px-3 py-2.5 transition-colors duration-200"
                      style={{
                        borderColor: isComplete || (isActive && !isLoading) ? "#C8942E33" : "#211A11",
                        background: isComplete ? "#1E140511" : isActive ? "#C8942E0A" : "#171008",
                      }}
                    >
                      <div
                        className="flex size-5 shrink-0 items-center justify-center rounded-full"
                        style={{
                          background: isComplete ? "#C8942E" : isActive ? "#30271A" : "#211A11",
                        }}
                      >
                        {isComplete ? (
                          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
                            <path d="M2 5l2.5 2.5L8 3" stroke="#000" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                          </svg>
                        ) : isLoading ? (
                          <Spinner size={10} color="#C8942E" />
                        ) : (
                          <span className="size-1.5 rounded-full" style={{ background: isActive ? "#C8942E" : "#6F6A62" }} />
                        )}
                      </div>
                      <span
                        className="font-mono text-xs"
                        style={{ color: isComplete || isActive ? "#F5F1E8" : "#6F6A62" }}
                      >
                        {label}
                      </span>
                      {isComplete && key === "detect" && detectedFormat && (
                        <span
                          className="ml-auto rounded font-mono text-xs px-1.5 py-0.5 uppercase"
                          style={{ background: "#C8942E22", color: "#C8942E" }}
                        >
                          {detectedFormat}
                        </span>
                      )}
                    </div>
                  )
                })}
              </div>

              {/* Output area */}
              <div
                className="rounded-md border"
                style={{ borderColor: "#211A11", background: "#171008", minHeight: "140px" }}
                aria-live="polite"
                aria-label="Pipeline output"
              >
                {stage === "idle" && (
                  <p className="p-4 font-mono text-xs" style={{ color: "#6F6A62" }}>
                    Output will appear here after running the pipeline.
                  </p>
                )}

                {stage === "error" && (
                  <div className="p-4">
                    <p className="font-mono text-xs" style={{ color: "#D95C4A" }}>
                      {errorMsg}
                    </p>
                  </div>
                )}

                {(stage === "done" || stage === "normalizing") && outputEvent && (
                  <pre
                    className="overflow-auto p-4 text-xs leading-relaxed"
                    style={{ fontFamily: "var(--font-mono)", color: "#F5F1E8", maxHeight: "300px" }}
                  >
                    {JSON.stringify(outputEvent, null, 2)}
                  </pre>
                )}

                {isRunning && stage !== "normalizing" && (
                  <p className="p-4 font-mono text-xs" style={{ color: "#6F6A62" }}>
                    {STAGE_LABELS[stage] ?? "Processing"}…
                  </p>
                )}
              </div>
            </div>
          </div>
        </div>

        <p className="mt-4 text-center text-xs" style={{ color: "#6F6A62" }}>
          Connected to the Helios Rust core pipeline running on Axum and Tokio.
        </p>
      </div>
    </section>
  )
}

function Spinner({ size = 12, color = "#F5F1E8" }: { size?: number; color?: string }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      className="animate-spin"
      aria-hidden="true"
    >
      <circle cx="8" cy="8" r="6" stroke={color} strokeWidth="2" opacity="0.2" />
      <path d="M8 2a6 6 0 0 1 6 6" stroke={color} strokeWidth="2" strokeLinecap="round" />
    </svg>
  )
}

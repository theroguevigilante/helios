type StatusType = "live" | "in-progress" | "planned"

interface PipelineNode {
  label: string
  status: StatusType
  mono?: boolean
}

const STATUS_CONFIG = {
  "live": { label: "Live", color: "#C8F05A", bg: "#C8F05A18" },
  "in-progress": { label: "In Progress", color: "#E0A33A", bg: "#E0A33A18" },
  "planned": { label: "Planned", color: "#66615A", bg: "#66615A18" },
} as const

const PARSER_NODES: PipelineNode[] = [
  { label: "Syslog Parser", status: "live", mono: true },
  { label: "JSON Parser", status: "live", mono: true },
  { label: "Apache Parser", status: "planned", mono: true },
  { label: "Nginx Parser", status: "planned", mono: true },
  { label: "CEF Parser", status: "planned", mono: true },
  { label: "LEEF Parser", status: "planned", mono: true },
]

const ENRICHMENT_NODES: PipelineNode[] = [
  { label: "Enrichment", status: "in-progress" },
  { label: "Rule Engine", status: "planned" },
  { label: "AI Classifier", status: "planned" },
]

function StatusPill({ status }: { status: StatusType }) {
  const cfg = STATUS_CONFIG[status]
  return (
    <span
      className="inline-flex items-center gap-1 rounded-full px-2 py-0.5 font-mono text-xs"
      style={{ background: cfg.bg, color: cfg.color }}
      title={cfg.label}
    >
      <span className="size-1.5 rounded-full" style={{ background: cfg.color }} aria-hidden="true" />
      {cfg.label}
    </span>
  )
}

function PipelineBox({ label, status, mono }: PipelineNode) {
  return (
    <div
      className="flex min-w-[120px] flex-col items-center gap-2 rounded-lg border p-3 text-center"
      style={{
        borderColor: status === "live" ? "#C8942E33" : "#30271A",
        background: status === "live" ? "#1E140520" : "#0D0A06",
      }}
    >
      <span
        className="text-xs font-medium leading-tight"
        style={{
          color: status === "live" ? "#F5F1E8" : status === "in-progress" ? "#A9A39A" : "#6F6A62",
          fontFamily: mono ? "var(--font-mono)" : "var(--font-sans)",
        }}
      >
        {label}
      </span>
      <StatusPill status={status} />
    </div>
  )
}

function Arrow({ vertical }: { vertical?: boolean }) {
  if (vertical) {
    return (
      <div className="flex justify-center py-1" aria-hidden="true">
        <svg width="16" height="20" viewBox="0 0 16 20" fill="none">
          <path d="M8 0v16M4 12l4 4 4-4" stroke="#30271A" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </div>
    )
  }
  return (
    <div className="flex shrink-0 items-center px-1" aria-hidden="true">
      <svg width="24" height="16" viewBox="0 0 24 16" fill="none">
        <path d="M0 8h20M16 4l4 4-4 4" stroke="#30271A" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </div>
  )
}

export function ArchitectureSection() {
  return (
    <section
      id="architecture"
      className="px-6 py-24"
      style={{ background: "#0D0A06" }}
    >
      <div className="mx-auto max-w-6xl">
        <div className="mb-12 text-center">
          <p className="mb-2 font-mono text-xs uppercase tracking-widest" style={{ color: "#C8942E" }}>
            Architecture
          </p>
          <h2
            className="mb-4 text-3xl font-bold tracking-tight md:text-4xl"
            style={{ color: "#F5F1E8", letterSpacing: "-0.02em" }}
          >
            How Helios Works
          </h2>
          <p className="mx-auto max-w-xl text-sm leading-relaxed" style={{ color: "#A9A39A" }}>
            A composable pipeline from raw log ingestion to analytics-ready events.
          </p>
        </div>

        {/* Legend */}
        <div className="mb-8 flex flex-wrap items-center justify-center gap-4">
          {(Object.entries(STATUS_CONFIG) as [StatusType, typeof STATUS_CONFIG[StatusType]][]).map(([key, cfg]) => (
            <div key={key} className="flex items-center gap-2">
              <span className="size-2 rounded-full" style={{ background: cfg.color }} aria-hidden="true" />
              <span className="font-mono text-xs" style={{ color: cfg.color }}>{cfg.label}</span>
            </div>
          ))}
        </div>

        {/* Pipeline diagram — horizontal on desktop, vertical on mobile */}
        <div className="overflow-x-auto">
          {/* Desktop layout */}
          <div className="hidden md:block">
            <div className="flex items-start justify-center gap-0 pb-4">
              {/* Source → Detector → Registry */}
              <PipelineBox label="Source Connectors" status="live" />
              <Arrow />
              <PipelineBox label="Format Detector" status="live" />
              <Arrow />
              <PipelineBox label="Parser Registry" status="live" />
              <Arrow />

              {/* Parser sub-column */}
              <div className="flex flex-col gap-1.5">
                {PARSER_NODES.map((n) => (
                  <PipelineBox key={n.label} {...n} />
                ))}
              </div>

              <Arrow />
              <PipelineBox label="Universal Event" status="live" />
              <Arrow />

              {/* Enrichment column */}
              <div className="flex flex-col gap-1.5">
                {ENRICHMENT_NODES.map((n) => (
                  <PipelineBox key={n.label} {...n} />
                ))}
              </div>

              <Arrow />
              <PipelineBox label="Storage" status="in-progress" />
              <Arrow />
              <PipelineBox label="API" status="in-progress" />
              <Arrow />
              <PipelineBox label="Visualization / SIEM" status="planned" />
            </div>
          </div>

          {/* Mobile layout — vertical */}
          <div className="flex flex-col items-center md:hidden">
            <PipelineBox label="Source Connectors" status="live" />
            <Arrow vertical />
            <PipelineBox label="Format Detector" status="live" />
            <Arrow vertical />
            <PipelineBox label="Parser Registry" status="live" />
            <Arrow vertical />
            <div
              className="mb-2 w-full max-w-xs rounded-lg border p-3"
              style={{ borderColor: "#30271A", background: "#171008" }}
            >
              <p className="mb-2 text-center font-mono text-xs" style={{ color: "#6F6A62" }}>Parsers</p>
              <div className="flex flex-wrap justify-center gap-2">
                {PARSER_NODES.map((n) => (
                  <PipelineBox key={n.label} {...n} />
                ))}
              </div>
            </div>
            <Arrow vertical />
            <PipelineBox label="Universal Event" status="live" />
            <Arrow vertical />
            <div
              className="mb-2 w-full max-w-xs rounded-lg border p-3"
              style={{ borderColor: "#30271A", background: "#171008" }}
            >
              <p className="mb-2 text-center font-mono text-xs" style={{ color: "#6F6A62" }}>Processing</p>
              <div className="flex flex-wrap justify-center gap-2">
                {ENRICHMENT_NODES.map((n) => (
                  <PipelineBox key={n.label} {...n} />
                ))}
              </div>
            </div>
            <Arrow vertical />
            <PipelineBox label="Storage" status="in-progress" />
            <Arrow vertical />
            <PipelineBox label="API" status="in-progress" />
            <Arrow vertical />
            <PipelineBox label="Visualization / SIEM" status="planned" />
          </div>
        </div>
      </div>
    </section>
  )
}

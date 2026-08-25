const TECH_BADGES = [
  "Rust",
  "Tokio",
  "Axum",
  "Serde",
  "syslog_loose",
  "SQLite (via sqlx)",
]

export function TechStackStrip() {
  return (
    <section id="tech-stack" className="px-6 py-16" style={{ background: "#0D0A06" }}>
      <div className="mx-auto max-w-5xl">
        <p className="mb-6 text-center font-mono text-xs uppercase tracking-widest" style={{ color: "#6F6A62" }}>
          Built With
        </p>
        <div className="flex flex-wrap items-center justify-center gap-3">
          {TECH_BADGES.map((tech) => (
            <span
              key={tech}
              className="rounded-md border px-4 py-2 font-mono text-sm transition-colors duration-150"
              style={{
                borderColor: "#30271A",
                background: "#171008",
                color: "#A9A39A",
              }}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLSpanElement).style.borderColor = "#C8942E55"
                ;(e.currentTarget as HTMLSpanElement).style.color = "#F5F1E8"
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLSpanElement).style.borderColor = "#30271A"
                ;(e.currentTarget as HTMLSpanElement).style.color = "#A9A39A"
              }}
            >
              {tech}
            </span>
          ))}
        </div>
      </div>
    </section>
  )
}

export function Footer() {
  return (
    <footer
      className="border-t px-6 py-12"
      style={{ borderColor: "#211A11", background: "#000000" }}
    >
      <div className="mx-auto max-w-5xl">
        <div className="flex flex-col items-center gap-4 text-center">
          <div className="flex items-center gap-2">
            <svg width="20" height="20" viewBox="0 0 48 48" fill="none" aria-hidden="true">
              <circle cx="24" cy="24" r="8" fill="#C8942E" />
              {[0, 45, 90, 135, 180, 225, 270, 315].map((angle, i) => {
                const rad = (angle * Math.PI) / 180
                const x1 = 24 + 12 * Math.cos(rad)
                const y1 = 24 + 12 * Math.sin(rad)
                const x2 = 24 + 20 * Math.cos(rad)
                const y2 = 24 + 20 * Math.sin(rad)
                return (
                  <line
                    key={i}
                    x1={x1} y1={y1} x2={x2} y2={y2}
                    stroke="#C8942E"
                    strokeWidth={i % 2 === 0 ? 2.5 : 1.5}
                    strokeLinecap="round"
                    opacity={i % 2 === 0 ? 1 : 0.5}
                  />
                )
              })}
            </svg>
            <span className="text-sm font-bold tracking-tight" style={{ color: "#F5F1E8" }}>
              Helios
            </span>
          </div>

          <a
            href="#"
            className="inline-flex items-center gap-2 rounded-md border px-4 py-2 text-sm font-mono transition-colors duration-150 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
            style={{ borderColor: "#30271A", background: "#171008", color: "#A9A39A" }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLAnchorElement).style.borderColor = "#C8942E55"
              ;(e.currentTarget as HTMLAnchorElement).style.color = "#F5F1E8"
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLAnchorElement).style.borderColor = "#30271A"
              ;(e.currentTarget as HTMLAnchorElement).style.color = "#A9A39A"
            }}
            aria-label="GitHub repository (placeholder — swap in the real repo URL)"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z" />
            </svg>
            GitHub
            <span className="font-sans text-xs" style={{ color: "#6F6A62" }}>
              (placeholder)
            </span>
          </a>

          <p className="text-xs" style={{ color: "#6F6A62" }}>
            An open-source Rust systems project, actively in development.
          </p>
        </div>
      </div>
    </footer>
  )
}

export function HeroSection() {
  return (
    <section
      id="hero"
      className="relative flex min-h-screen flex-col items-center justify-center overflow-hidden px-6 py-24 text-center"
      style={{ background: "#000000" }}
    >
      {/* Subtle radial glow */}
      <div
        className="pointer-events-none absolute inset-0 opacity-20"
        style={{
          background:
            "radial-gradient(ellipse 60% 40% at 50% 60%, #C8942E22 0%, transparent 70%)",
        }}
        aria-hidden="true"
      />

      <div className="relative z-10 mx-auto max-w-4xl">
        {/* Wordmark */}
        <div className="mb-6 flex items-center justify-center gap-3">
          <HeliosLogo />
          <span
            className="text-5xl font-bold tracking-tight md:text-7xl"
            style={{ color: "#F5F1E8", fontFamily: "var(--font-sans)", letterSpacing: "-0.03em" }}
          >
            Helios
          </span>
        </div>

        {/* Tagline */}
        <p
          className="mb-4 text-2xl font-semibold tracking-tight md:text-3xl"
          style={{ color: "#F5F1E8", letterSpacing: "-0.02em" }}
        >
          One pipeline.{" "}
          <span style={{ color: "#C8942E" }}>Any log format.</span>{" "}
          Zero rewrites.
        </p>

        {/* Subhead */}
        <p
          className="mx-auto mb-10 max-w-2xl text-base leading-relaxed md:text-lg"
          style={{ color: "#A9A39A" }}
        >
          Helios normalizes heterogeneous logs — syslog, JSON, and more in development —
          into a single lossless schema for SIEM, threat hunting, and security analytics platforms.
        </p>

        {/* CTA */}
        <a
          href="#pipeline-demo"
          className="inline-flex items-center gap-2 rounded-md px-6 py-3 text-sm font-semibold transition-colors duration-150 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
          style={{
            background: "#C8942E",
            color: "#000000",
            fontFamily: "var(--font-sans)",
          }}
          onMouseEnter={(e) => { (e.currentTarget as HTMLAnchorElement).style.background = "#E7B84B" }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLAnchorElement).style.background = "#C8942E" }}
        >
          Try the live pipeline demo
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M3 8h10M9 4l4 4-4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </a>

        {/* Badge row */}
        <div className="mt-8 flex flex-wrap items-center justify-center gap-3">
          {[
            { label: "Rust", icon: <RustIcon /> },
            { label: "Built on syslog_loose, Axum, Tokio" },
            { label: "Open source, actively in development", dot: true },
          ].map((badge, i) => (
            <span
              key={i}
              className="inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-mono"
              style={{
                borderColor: "#30271A",
                background: "#0D0A06",
                color: "#A9A39A",
              }}
            >
              {badge.dot && (
                <span className="inline-block size-1.5 rounded-full" style={{ background: "#C8F05A" }} />
              )}
              {badge.icon}
              {badge.label}
            </span>
          ))}
        </div>
      </div>

      {/* Scroll indicator */}
      <div className="absolute bottom-8 left-1/2 -translate-x-1/2 animate-bounce" aria-hidden="true">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
          <path d="M5 8l5 5 5-5" stroke="#6F6A62" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </div>
    </section>
  )
}

function HeliosLogo() {
  return (
    <svg width="48" height="48" viewBox="0 0 48 48" fill="none" aria-hidden="true">
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
  )
}

function RustIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" stroke="currentColor" strokeWidth="2" fill="none" strokeLinejoin="round" />
    </svg>
  )
}

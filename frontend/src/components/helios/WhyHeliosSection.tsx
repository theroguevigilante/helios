interface WhyCard {
  title: string
  body: string
  icon: React.ReactNode
}

const CARDS: WhyCard[] = [
  {
    title: "Reuse over reinvent",
    body: "Built on proven crates like syslog_loose instead of hand-rolled parsers, wherever mature libraries already exist.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M4 7h16M4 12h16M4 17h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
        <circle cx="19" cy="17" r="3" stroke="currentColor" strokeWidth="1.5" />
        <path d="M21.5 19.5L23 21" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      </svg>
    ),
  },
  {
    title: "Lossless by design",
    body: "The raw event is always preserved alongside the normalized one, so nothing is lost in translation.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M12 3l8 4v5c0 5-3.5 8-8 9-4.5-1-8-4-8-9V7l8-4z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round" />
        <path d="M9 12l2 2 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    ),
  },
  {
    title: "Composable",
    body: "A plugin-style parser registry lets you add a new format without touching the core pipeline.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <rect x="3" y="3" width="7" height="7" rx="1" stroke="currentColor" strokeWidth="1.5" />
        <rect x="14" y="3" width="7" height="7" rx="1" stroke="currentColor" strokeWidth="1.5" />
        <rect x="3" y="14" width="7" height="7" rx="1" stroke="currentColor" strokeWidth="1.5" />
        <path d="M17.5 14v7M14 17.5h7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      </svg>
    ),
  },
  {
    title: "Air-gap friendly",
    body: "Fully offline-capable with no cloud dependency. Helios runs entirely within your own network.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <circle cx="12" cy="12" r="9" stroke="currentColor" strokeWidth="1.5" />
        <path d="M3 12h18M12 3c2.5 2.5 4 5.5 4 9s-1.5 6.5-4 9c-2.5-2.5-4-5.5-4-9s1.5-6.5 4-9z" stroke="currentColor" strokeWidth="1.5" />
      </svg>
    ),
  },
]

export function WhyHeliosSection() {
  return (
    <section id="why" className="px-6 py-24" style={{ background: "#000000" }}>
      <div className="mx-auto max-w-6xl">
        <div className="mb-12 text-center">
          <p className="mb-2 font-mono text-xs uppercase tracking-widest" style={{ color: "#C8942E" }}>
            Why Helios
          </p>
          <h2
            className="mb-4 text-3xl font-bold tracking-tight md:text-4xl"
            style={{ color: "#F5F1E8", letterSpacing: "-0.02em" }}
          >
            Built for Security Engineers
          </h2>
        </div>

        <div className="grid grid-cols-2 gap-4">
          {CARDS.map((card) => (
            <div
              key={card.title}
              className="group flex flex-col gap-4 rounded-xl border p-6 transition-colors duration-200"
              style={{ borderColor: "#30271A", background: "#0D0A06" }}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLDivElement).style.borderColor = "#C8942E55"
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLDivElement).style.borderColor = "#30271A"
              }}
            >
              <div
                className="flex size-11 items-center justify-center rounded-lg transition-colors duration-200"
                style={{ background: "#1E1405", color: "#C8942E" }}
              >
                {card.icon}
              </div>
              <h3 className="text-base font-semibold" style={{ color: "#F5F1E8" }}>
                {card.title}
              </h3>
              <p className="text-sm leading-relaxed" style={{ color: "#A9A39A" }}>
                {card.body}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}

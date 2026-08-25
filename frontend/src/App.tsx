import { ArchitectureSection } from "@/components/helios/ArchitectureSection"
import { Footer, TechStackStrip } from "@/components/helios/TechStackFooter"
import { HeroSection } from "@/components/helios/HeroSection"
import { PipelineDemo } from "@/components/helios/PipelineDemo"
import { WhyHeliosSection } from "@/components/helios/WhyHeliosSection"

function App() {
  return (
    <div className="min-h-screen" style={{ background: "#000000", color: "#F5F1E8" }}>
      <header
        className="fixed top-0 right-0 left-0 z-50 border-b backdrop-blur-md"
        style={{ background: "#000000CC", borderColor: "#211A11" }}
      >
        <nav className="mx-auto flex max-w-6xl items-center justify-between px-6 py-3" aria-label="Main navigation">
          <a href="#hero" className="flex items-center gap-2" aria-label="Helios home">
            <span className="size-2 rounded-full" style={{ background: "#C8942E" }} aria-hidden="true" />
            <span className="text-sm font-bold tracking-tight" style={{ color: "#F5F1E8" }}>Helios</span>
          </a>
          <div className="hidden items-center gap-6 sm:flex">
            <a href="#pipeline-demo" className="text-xs transition-colors hover:text-[#C8942E]" style={{ color: "#A9A39A" }}>Demo</a>
            <a href="#architecture" className="text-xs transition-colors hover:text-[#C8942E]" style={{ color: "#A9A39A" }}>Architecture</a>
            <a href="#why" className="text-xs transition-colors hover:text-[#C8942E]" style={{ color: "#A9A39A" }}>Why Helios</a>
          </div>
          <a href="#pipeline-demo" className="font-mono text-xs" style={{ color: "#C8942E" }}>Try demo →</a>
        </nav>
      </header>

      <main>
        <HeroSection />
        <PipelineDemo />
        <ArchitectureSection />
        <WhyHeliosSection />
        <TechStackStrip />
      </main>
      <Footer />
    </div>
  )
}

export default App

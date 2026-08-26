<script lang="ts">
  import { onMount } from 'svelte';

  let visible = $state(false);

  onMount(() => {
    visible = true;
  });

  const features = [
    {
      icon: '⚡',
      title: 'Auto-Detection',
      desc: 'Automatically identifies log formats — Syslog, CEF, JSON, and more — without manual configuration.'
    },
    {
      icon: '🔄',
      title: 'Universal Schema',
      desc: 'Normalizes every log into a unified event model inspired by ECS and OpenTelemetry standards.'
    },
    {
      icon: '🔒',
      title: 'Lossless Preservation',
      desc: 'Preserves the original raw event alongside the normalized data for forensic traceability.'
    },
    {
      icon: '🚀',
      title: '2.4MB Binary, 4MB RAM',
      desc: 'Built in Rust. Deploys on a Raspberry Pi to a datacenter. No JVM, no runtime dependencies.'
    },
    {
      icon: '🔌',
      title: 'Plugin Architecture',
      desc: 'Add new log sources by dropping in a parser crate. No modifications to existing code required.'
    },
    {
      icon: '🤖',
      title: 'AI-Ready Output',
      desc: 'Structured, normalized JSON events ready for SIEM ingestion, ML pipelines, and threat hunting.'
    }
  ];
</script>

<svelte:head>
  <title>Helios — Universal Log Pre-processing Platform</title>
</svelte:head>

<div class="relative overflow-hidden">
  <!-- Hero -->
  <section class="relative flex flex-col items-center justify-center min-h-screen px-6 text-center">
    <!-- Glow effect -->
    <div class="absolute top-1/3 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-helios-accent/5 rounded-full blur-[150px] pointer-events-none"></div>

    <div class="relative z-10 max-w-3xl transition-all duration-1000 {visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'}">
      <div class="inline-flex items-center gap-2 px-4 py-1.5 mb-8 text-sm border rounded-full border-helios-border text-helios-muted bg-helios-surface">
        <span class="w-2 h-2 rounded-full bg-helios-green animate-pulse"></span>
        Smart India Hackathon 2026
      </div>

      <h1 class="text-5xl font-bold tracking-tight sm:text-7xl">
        <span class="text-helios-accent">Helios</span>
      </h1>

      <p class="mt-4 text-xl text-helios-muted sm:text-2xl">
        Universal Log Pre-processing &amp; Observability Platform
      </p>

      <p class="max-w-xl mx-auto mt-6 text-helios-muted">
        Ingest logs from any source. Detect the format automatically.
        Normalize into a universal schema. Search, visualize, and analyze — all from one platform.
      </p>

      <div class="flex flex-wrap items-center justify-center gap-4 mt-10">
        <a
          href="/dashboard"
          class="px-6 py-3 text-sm font-semibold text-black transition rounded-lg bg-helios-accent hover:bg-helios-accent/90"
        >
          Open Dashboard →
        </a>
        <a
          href="https://github.com/theroguevigilante/helios"
          target="_blank"
          rel="noopener"
          class="px-6 py-3 text-sm font-semibold transition border rounded-lg border-helios-border text-helios-text hover:bg-helios-surface-2"
        >
          View Source
        </a>
      </div>
    </div>

    <!-- Scroll indicator -->
    <div class="absolute bottom-8 animate-bounce text-helios-muted">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
      </svg>
    </div>
  </section>

  <!-- Pipeline visualization -->
  <section class="px-6 py-20">
    <div class="max-w-4xl mx-auto">
      <h2 class="mb-12 text-3xl font-bold text-center">How It Works</h2>
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-5">
        {#each ['Ingest', 'Detect', 'Parse', 'Enrich', 'Store'] as step, i}
          <div class="flex flex-col items-center gap-2">
            <div class="flex items-center justify-center w-14 h-14 text-lg font-bold rounded-xl bg-helios-surface-2 border border-helios-border text-helios-accent">
              {i + 1}
            </div>
            <span class="text-sm font-medium">{step}</span>
          </div>
          {#if i < 4}
            <div class="hidden sm:flex items-center justify-center text-helios-muted">→</div>
          {/if}
        {/each}
      </div>
    </div>
  </section>

  <!-- Features grid -->
  <section class="px-6 py-20">
    <div class="max-w-5xl mx-auto">
      <h2 class="mb-12 text-3xl font-bold text-center">Built for Production</h2>
      <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {#each features as feature}
          <div class="p-6 transition border rounded-xl border-helios-border bg-helios-surface hover:border-helios-accent/30">
            <div class="mb-3 text-2xl">{feature.icon}</div>
            <h3 class="mb-2 text-lg font-semibold">{feature.title}</h3>
            <p class="text-sm text-helios-muted">{feature.desc}</p>
          </div>
        {/each}
      </div>
    </div>
  </section>

  <!-- Footer -->
  <footer class="px-6 py-12 text-sm text-center border-t border-helios-border text-helios-muted">
    Helios • Smart India Hackathon 2026 • Built with Rust &amp; Svelte
  </footer>
</div>

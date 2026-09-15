<script lang="ts">
  import { onMount } from 'svelte';

  let visible = $state(false);
  let logs = $state<string[]>([]);
  let metricsVisible = $state(false);
  let splashText = $state("");

  const splashTexts = [
    "Built from day one for Edge & IoT!",
    "Designed with air-gapping in mind!",
    "No JVM. No bloat. Pure Metal.",
    "Unapologetically Rusty.",
    "Written in Rust, btw.",
    "Also runs on a Raspberry Pi!",
    "Fearless Concurrency.",
    "Ingests logs like a black hole.",
    "Goodbye, browser freezes.",
    "0 CFG. 100% Magic."
  ];

  const sampleLogs = [
    "[SYS] Initializing Universal FormatDetector...",
    "[INFO] Raw Syslog stream detected. Normalizing.",
    "[DEBUG] Processing Nginx access logs...",
    "[WARN] EVTX Binary detected. Engaging native parser.",
    "[CRIT] Authentication failure anomaly detected on sshd.",
    "[SYS] Engine memory footprint: 4.2MB. CPU: 2%.",
    "[INFO] Routing 1,000,000 normalized events to UI."
  ];

  onMount(() => {
    splashText = splashTexts[Math.floor(Math.random() * splashTexts.length)];
    setTimeout(() => visible = true, 100);
    setTimeout(() => metricsVisible = true, 800);

    let i = 0;
    const interval = setInterval(() => {
      if (logs.length >= 8) logs = logs.slice(1);
      const hex = Math.floor(Math.random() * 0xFFFFFF).toString(16).toUpperCase().padStart(6, '0');
      logs = [...logs, `[0x${hex}] ${sampleLogs[i % sampleLogs.length]}`];
      i++;
    }, 600);

    return () => clearInterval(interval);
  });
</script>

<svelte:head>
  <title>Helios | Forensic Log Analysis Engine</title>
</svelte:head>

<div class="relative min-h-screen bg-helios-bg overflow-hidden font-sans selection:bg-helios-accent/30 selection:text-helios-accent">
  
  <!-- Glowing Grid Background -->
  <div class="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:24px_24px]"></div>
  <div class="absolute left-0 right-0 top-0 -z-10 m-auto h-[310px] w-[310px] rounded-full bg-helios-accent/20 opacity-50 blur-[100px]"></div>
  <div class="absolute bottom-0 right-0 -z-10 m-auto h-[400px] w-[400px] rounded-full bg-blue-600/20 opacity-40 blur-[120px]"></div>

  <!-- Header -->
  <header class="relative z-10 flex items-center justify-between px-8 py-6">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-helios-accent to-blue-600 shadow-lg shadow-helios-accent/20 flex items-center justify-center">
        <svg class="w-6 h-6 text-helios-text" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
      </div>
      <span class="text-xl font-bold tracking-wider text-helios-text">HELIOS</span>
    </div>
    
  </header>

  <!-- Hero Section -->
  <main class="relative z-10 flex flex-col items-center justify-center px-6 pt-20 pb-32 text-center lg:pt-32">
    
    <!-- Animated Terminal Window -->
    <div class="mb-12 w-full max-w-2xl text-left border border-helios-border/50 bg-helios-surface/80 backdrop-blur-md rounded-lg shadow-2xl transition-all duration-1000 transform {visible ? 'opacity-100 translate-y-0 scale-100' : 'opacity-0 translate-y-8 scale-95'}">
      <div class="flex items-center gap-2 px-4 py-2 border-b border-helios-border/50 bg-helios-surface-2/50 rounded-t-lg relative overflow-visible">
        <div class="w-3 h-3 rounded-full bg-red-500/80"></div>
        <div class="w-3 h-3 rounded-full bg-amber-500/80"></div>
        <div class="w-3 h-3 rounded-full bg-green-500/80"></div>
        <div class="ml-2 text-[10px] font-mono text-helios-muted tracking-widest uppercase">
          rust-engine
        </div>
        <div class="ml-auto relative">
          {#if splashText}
            <div class="absolute -top-1 right-[-7px] md:right-[-39px] rotate-[15deg] text-yellow-400 font-extrabold text-sm md:text-base drop-shadow-[0_0_8px_rgba(250,204,21,0.8)] animate-pulse whitespace-nowrap z-20 pointer-events-none normal-case tracking-normal">
              {splashText}
            </div>
          {/if}
        </div>
      </div>
      <div class="p-4 h-48 font-mono text-xs text-helios-muted flex flex-col justify-end overflow-hidden rounded-b-lg">
        {#each logs as log}
          <div class="mb-1 text-green-400/80 drop-shadow-[0_0_8px_rgba(74,222,128,0.4)]">
            <span class="text-helios-muted/50">{'>'}</span> {log}
          </div>
        {/each}
        <div class="animate-pulse w-2 h-4 bg-helios-accent mt-1"></div>
      </div>
    </div>

    <!-- Main Copy -->
    <div class="max-w-4xl transition-all duration-1000 delay-300 {visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'}">
      <div class="relative inline-block">
        <h1 class="text-5xl font-extrabold tracking-tight text-helios-text sm:text-7xl lg:text-8xl">
          Forensics at the <br/>
          <span class="text-transparent bg-clip-text bg-gradient-to-r from-helios-accent to-blue-400 drop-shadow-[0_0_20px_rgba(56,189,248,0.4)]">
            Speed of Light.
          </span>
        </h1>
      </div>
      <p class="max-w-2xl mx-auto mt-8 text-lg sm:text-xl text-helios-muted/80 leading-relaxed font-light">
        The Swiss Army Knife of Log Preprocessing.
      </p>

      <!-- CTA -->
      <div class="mt-12 flex items-center justify-center gap-6">
        <a href="/dashboard" class="group relative px-8 py-4 font-bold text-black uppercase tracking-wider bg-helios-accent rounded-lg overflow-hidden transition-transform hover:scale-105 active:scale-95 shadow-[0_0_40px_rgba(56,189,248,0.4)] hover:shadow-[0_0_60px_rgba(56,189,248,0.6)]">
          <div class="absolute inset-0 bg-white/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out"></div>
          <span class="relative flex items-center gap-2">
            Initialize Engine
            <svg class="w-5 h-5 transition-transform group-hover:translate-x-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" /></svg>
          </span>
        </a>
      </div>
    </div>

    <!-- Crazy Stats -->
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-8 mt-24 max-w-5xl w-full transition-all duration-1000 delay-500 {metricsVisible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-12'}">
      <div class="flex flex-col items-center p-6 border border-helios-border/30 rounded-2xl bg-helios-surface/20 backdrop-blur">
        <div class="text-4xl font-bold text-helios-text mb-2 font-mono drop-shadow-[0_0_15px_rgba(255,255,255,0.3)]">∞</div>
        <div class="text-sm font-semibold tracking-widest text-helios-accent uppercase">Infinite DOM</div>
        <div class="text-xs text-helios-muted mt-2 text-center">Zero browser freezing. Render 1,000,000+ logs instantly via pure DOM virtualization.</div>
      </div>
      <div class="flex flex-col items-center p-6 border border-helios-border/30 rounded-2xl bg-helios-surface/20 backdrop-blur">
        <div class="text-4xl font-bold text-helios-text mb-2 font-mono drop-shadow-[0_0_15px_rgba(255,255,255,0.3)]">4<span class="text-2xl text-helios-muted">MB</span></div>
        <div class="text-sm font-semibold tracking-widest text-helios-accent uppercase">Forged in Rust</div>
        <div class="text-xs text-helios-muted mt-2 text-center">Micro-binary architecture. No JVM. No bloat. Pure native metal processing speeds.</div>
      </div>
      <div class="flex flex-col items-center p-6 border border-helios-border/30 rounded-2xl bg-helios-surface/20 backdrop-blur">
        <div class="text-4xl font-bold text-helios-text mb-2 font-mono drop-shadow-[0_0_15px_rgba(255,255,255,0.3)]">0<span class="text-2xl text-helios-muted">CFG</span></div>
        <div class="text-sm font-semibold tracking-widest text-helios-accent uppercase">Auto-Detect</div>
        <div class="text-xs text-helios-muted mt-2 text-center">Drag and drop any format—Syslog, JSON, CEF, or EVTX. The smart registry detects and normalizes it on the fly.</div>
      </div>
    </div>
  </main>
</div>

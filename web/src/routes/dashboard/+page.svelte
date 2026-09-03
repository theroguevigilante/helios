<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import TimelineChart from '$lib/components/TimelineChart.svelte';
  import SeverityChart from '$lib/components/SeverityChart.svelte';
  import Sidebar from '$lib/components/Layout/Sidebar.svelte';
  import LogInspector from '$lib/components/Layout/LogInspector.svelte';
  import type { ParsedEvent } from '$lib/types';
  import { createVirtualizer } from '@tanstack/svelte-virtual';

  let events = $state<ParsedEvent[]>([]);
  let isConnected = $state(false);
  let isPaused = $state(false);
  let displayTimezone = $state<'Local' | 'UTC'>('Local');
  let queuedEvents: ParsedEvent[] = [];
  
  let searchQuery = $state('');
  let selectedSeverity = $state('ALL');
  let timeRange = $state<[string, string] | null>(null);
  
  let eventCount = $derived(events.length);
  let filteredEvents = $derived(events.filter(e => {
    if (selectedSeverity !== 'ALL' && e.severity !== selectedSeverity) return false;
    if (timeRange) {
      const t = e.timestamp.slice(0, 19);
      if (t < timeRange[0] || t > timeRange[1]) return false;
    }
    if (searchQuery) {
      // Allow advanced query "service:sshd"
      if (searchQuery.includes(':')) {
        const [field, value] = searchQuery.split(':');
        const q = value.toLowerCase();
        if (field.toLowerCase() === 'service' && e.service?.toLowerCase() !== q) return false;
        if (field.toLowerCase() === 'host' && e.hostname?.toLowerCase() !== q) return false;
      } else {
        const q = searchQuery.toLowerCase();
        return (e.message.toLowerCase().includes(q) || 
                (e.hostname && e.hostname.toLowerCase().includes(q)) || 
                (e.service && e.service.toLowerCase().includes(q)));
      }
    }
    return true;
  }));

  // Stats
  let eventsPerSec = $state(0);
  let parserCount = $state(0);
  let _lastCount = 0;
  let statsInterval: number;

  let selectedEvent = $state<ParsedEvent | null>(null);
    let uploadWarning = $state('');
  
  let eventSource: EventSource | null = null;
  let virtualListEl: HTMLDivElement | null = $state(null);

  // Virtualizer setup
  let virtualizer = $derived(createVirtualizer({
    get count() { return filteredEvents.length; },
    getScrollElement: () => virtualListEl,
    estimateSize: () => 36,
    overscan: 20,
  }));

  onMount(() => {
    connectStream();
    
    statsInterval = setInterval(() => {
      eventsPerSec = events.length - _lastCount;
      _lastCount = events.length;
    }, 1000);

    fetch('http://localhost:8080/api/v1/stats')
      .then(r => r.json())
      .then(d => {
        parserCount = d.parsers.length;
      })
      .catch(console.error);
  });

  onDestroy(() => {
    if (eventSource) eventSource.close();
    clearInterval(statsInterval);
  });

  function connectStream() {
    if (eventSource) eventSource.close();
    eventSource = new EventSource('http://localhost:8080/api/v1/stream');
    
    eventSource.onopen = () => {
      isConnected = true;
    };
    
    eventSource.onerror = () => {
      isConnected = false;
    };

    eventSource.onmessage = (e) => {
      const newEvent: ParsedEvent = JSON.parse(e.data);
      if (isPaused) {
        queuedEvents.push(newEvent);
      } else {
        events = [newEvent, ...events];
      }
    };
  }

  function togglePause() {
    isPaused = !isPaused;
    if (!isPaused && queuedEvents.length > 0) {
      // Flush queued events
      events = [...queuedEvents.reverse(), ...events];
      queuedEvents = [];
    }
  }

  function clearStream() {
    events = [];
    selectedEvent = null;
  }


  async function uploadFile(file: File) {
    const formData = new FormData();
    formData.append('file', file);
    try {
      const res = await fetch('http://localhost:8080/api/v1/upload', {
        method: 'POST',
        body: formData
      });
      if (!res.ok) {
        console.error("Upload failed", await res.text());
      } else {
        const data = await res.json();
        // Prepend the new parsed events from the uploaded file
        events = [...data, ...events];
      }
    } catch (e) {
      console.error("Upload error", e);
    }
  }

  function downloadJSON() {
    const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(filteredEvents, null, 2));
    const dlAnchorElem = document.createElement('a');
    dlAnchorElem.setAttribute("href", dataStr);
    dlAnchorElem.setAttribute("download", "helios_forensic_export.json");
    dlAnchorElem.click();
  }

  function severityBg(sev: string | null): string {
    switch (sev) {
      case 'CRIT': return 'bg-red-500/10 border-red-500/30 text-red-400';
      case 'ERROR': return 'bg-red-400/10 border-red-400/30 text-red-400';
      case 'WARN': return 'bg-amber-400/10 border-amber-400/30 text-amber-400';
      case 'NOTICE': return 'bg-blue-400/10 border-blue-400/30 text-blue-400';
      case 'INFO': return 'bg-green-400/10 border-green-400/30 text-green-400';
      case 'DEBUG': return 'bg-gray-400/10 border-gray-400/30 text-gray-400';
      default: return 'bg-helios-surface-2 border-helios-border text-helios-muted';
    }
  }
</script>

<svelte:head>
  <title>Helios Forensic Dashboard</title>
</svelte:head>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="relative flex flex-col h-screen text-helios-text bg-helios-bg font-sans">
  <!-- Top bar -->
  <header class="flex items-center justify-between px-6 py-3 border-b border-helios-border bg-helios-surface shadow-sm shrink-0">
    <div class="flex items-center gap-3">
      <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-helios-accent to-blue-600 shadow-lg shadow-helios-accent/20 flex items-center justify-center">
        <svg class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
      </div>
      <div>
        <h1 class="text-lg font-bold text-helios-text leading-tight">Helios Forensic</h1>
        <p class="text-[10px] uppercase font-bold tracking-widest text-helios-accent">Analysis Engine</p>
      </div>
    </div>
    
    <div class="flex items-center gap-4 text-sm">
      <div class="flex items-center gap-2">
        <select bind:value={displayTimezone} class="px-2 py-1 text-xs font-semibold rounded bg-helios-surface-2 border border-helios-border text-helios-text focus:outline-none focus:border-helios-accent/50 cursor-pointer">
          <option value="Local">Local Time</option>
          <option value="UTC">UTC</option>
        </select>
        <div class="relative group flex items-center justify-center cursor-help">
          <div class="w-5 h-5 rounded-full border border-helios-border text-helios-muted hover:text-helios-accent hover:border-helios-accent flex items-center justify-center text-xs font-bold transition-colors">?</div>
          <div class="absolute top-full right-0 mt-2 w-64 p-3 text-xs rounded shadow-xl bg-helios-surface border border-helios-border text-helios-text opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-50">
            <strong>Timezone Converter</strong><br/>
            Logs are ingested and stored in UTC. This toggle changes how timestamps are displayed across the dashboard (Charts, Grid, and Inspector).
          </div>
        </div>
      </div>
      
      <div class="h-6 w-px bg-helios-border"></div>
      
      <div class="flex items-center gap-2 px-3 py-1.5 rounded bg-helios-surface-2 border border-helios-border">
        <span class="w-2 h-2 rounded-full {isConnected ? 'bg-helios-green animate-pulse' : 'bg-helios-red'}"></span>
        <span class="text-xs font-semibold text-helios-muted uppercase tracking-wider">{isConnected ? 'Live' : 'Offline'}</span>
      </div>
      
      <div class="h-6 w-px bg-helios-border"></div>
      
      <div class="flex gap-2">
        <button onclick={togglePause} class="px-3 py-1.5 text-xs font-semibold rounded bg-helios-surface-2 border border-helios-border hover:bg-helios-border text-helios-text transition-colors flex items-center gap-2">
          {#if isPaused}
            <svg class="w-3 h-3 text-helios-green" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg> Resume
          {:else}
            <svg class="w-3 h-3 text-helios-accent" fill="currentColor" viewBox="0 0 24 24"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg> Pause
          {/if}
        </button>
        <button onclick={clearStream} class="px-3 py-1.5 text-xs font-semibold rounded bg-helios-surface-2 border border-helios-border hover:bg-helios-border text-helios-text transition-colors flex items-center gap-2">
          <svg class="w-3 h-3 text-helios-red" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg> Clear
        </button>
        <button onclick={downloadJSON} class="px-3 py-1.5 text-xs font-semibold rounded bg-helios-surface-2 border border-helios-border hover:bg-helios-border text-helios-text transition-colors flex items-center gap-2">
          <svg class="w-3 h-3 text-helios-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" /></svg> Export
        </button>
      </div>
      
      <div class="h-6 w-px bg-helios-border"></div>

      <div class="flex items-center gap-4 text-xs font-mono text-helios-muted">
        <div>Total: <span class="text-helios-text">{eventCount}</span></div>
        <div>Rate: <span class="text-helios-text">{eventsPerSec}</span>/s</div>
        <div>Parsers: <span class="text-helios-text">{parserCount}</span></div>
      </div>
    </div>
  </header>

  <!-- Main IDE Layout -->
  <div class="flex flex-1 overflow-hidden">
    <!-- Left Sidebar (Filters) -->
    <Sidebar bind:searchQuery bind:selectedSeverity onFileUpload={(file) => {
      if (file.size > 5 * 1024 * 1024) {
        uploadWarning = `Warning: Large file detected (${(file.size / 1024 / 1024).toFixed(1)}MB). The virtualized grid can handle it, but initial parsing may take a moment.`;
      } else {
        uploadWarning = '';
      }
      uploadFile(file);
    }} />

    <!-- Center Main Content -->
    <main class="flex flex-col flex-1 overflow-hidden relative bg-helios-bg">
      {#if uploadWarning}
        <div class="m-4 p-4 text-sm font-semibold border rounded-lg bg-amber-500/10 border-amber-500/30 text-amber-400 flex-shrink-0">
          <div class="flex justify-between items-start">
            <span>{uploadWarning}</span>
            <button onclick={() => uploadWarning = ''} class="text-amber-500/70 hover:text-amber-500 ml-4 shrink-0" aria-label="Dismiss warning">
              <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      {/if}

      <!-- Top Half: Charts -->
      <div class="grid grid-cols-1 gap-4 p-4 lg:grid-cols-2 shrink-0 border-b border-helios-border">
        <div class="h-64 p-4 border rounded-xl border-helios-border bg-helios-surface shadow-sm">
          <h3 class="mb-3 text-xs font-bold tracking-wider text-helios-muted uppercase">Events Over Time</h3>
          <TimelineChart events={filteredEvents} {displayTimezone} onTimeRangeSelect={(r) => timeRange = r} />
        </div>
        <div class="h-64 p-4 border rounded-xl border-helios-border bg-helios-surface shadow-sm">
          <h3 class="mb-3 text-xs font-bold tracking-wider text-helios-muted uppercase">Severity Distribution</h3>
          <SeverityChart events={filteredEvents} />
        </div>
      </div>

      <!-- Bottom Half: Virtualized Data Grid -->
      <div class="flex flex-col flex-1 overflow-hidden">
        <!-- Table Header (Fixed) -->
        <div class="flex border-b border-helios-border bg-helios-surface-2 text-xs font-bold tracking-wider text-helios-muted uppercase shrink-0 px-4">
          <div class="py-3 w-44 shrink-0">Timestamp</div>
          <div class="py-3 w-24 shrink-0">Severity</div>
          <div class="py-3 w-40 shrink-0">Host</div>
          <div class="py-3 w-36 shrink-0">Service</div>
          <div class="py-3 flex-1">Message</div>
        </div>
        
        <!-- Virtualized Table Body -->
        <div 
          bind:this={virtualListEl} 
          class="flex-1 overflow-y-auto w-full relative"
        >
          <div style="height: {$virtualizer.getTotalSize()}px; width: 100%; position: relative;">
            {#each $virtualizer.getVirtualItems() as virtualRow (virtualRow.index)}
              {@const event = filteredEvents[virtualRow.index]}
              <div 
                class="absolute top-0 left-0 w-full flex items-center px-4 border-b border-helios-border/50 hover:bg-helios-surface-2/50 cursor-pointer transition-colors text-sm"
                style="height: {virtualRow.size}px; transform: translateY({virtualRow.start}px);"
                onclick={() => selectedEvent = event} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && (selectedEvent = event)}
              >
                <div class="w-44 shrink-0 font-mono text-xs text-helios-muted truncate pr-2">
                  {displayTimezone === 'UTC' ? new Date(event.timestamp).toLocaleTimeString('en-US', { timeZone: 'UTC', hour12: false }) : new Date(event.timestamp).toLocaleTimeString()}
                </div>
                <div class="w-24 shrink-0 pr-2">
                  <span class="px-2 py-0.5 text-[10px] uppercase font-bold border rounded {severityBg(event.severity)}">
                    {event.severity ?? '—'}
                  </span>
                </div>
                <div class="w-40 shrink-0 font-mono text-xs truncate pr-2" title={event.hostname ?? ''}>
                  {event.hostname ?? '—'}
                </div>
                <div class="w-36 shrink-0 font-mono text-xs text-helios-accent truncate pr-2" title={event.service ?? ''}>
                  {event.service ?? '—'}
                </div>
                <div class="flex-1 text-xs truncate" title={event.message}>
                  {event.message}
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </main>

    <!-- Right Drawer (Log Inspector) -->
    <LogInspector event={selectedEvent} {displayTimezone} onClose={() => selectedEvent = null} />
  </div>
</div>

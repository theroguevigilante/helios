<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import SeverityChart from '$lib/components/SeverityChart.svelte';
  import TimelineChart from '$lib/components/TimelineChart.svelte';

  interface ParsedEvent {
    timestamp: string;
    hostname: string | null;
    service: string | null;
    severity: string | null;
    message: string;
    raw_event: string;
  }

  let events = $state<ParsedEvent[]>([]);
  let searchQuery = $state('');
  let selectedSeverity = $state('ALL');
  let isConnected = $state(false);
  let eventCount = $state(0);
  let eventsPerSec = $state(0);
  let parserCount = $state(13);
  let selectedEvent = $state<ParsedEvent | null>(null);
  let isDragging = $state(false);
  let uploadWarning = $state('');
  
  let eventSource: EventSource | null = null;

  onMount(() => {
    eventSource = new EventSource('http://localhost:8080/api/v1/stream');
    
    eventSource.onopen = () => {
      isConnected = true;
    };
    
    eventSource.onmessage = (e) => {
      try {
        const ev = JSON.parse(e.data);
        events = [ev, ...events].slice(0, 1000);
        eventCount++;
      } catch (err) {
        console.error(err);
      }
    };

    eventSource.onerror = () => {
      isConnected = false;
    };

    // Calculate events per second roughly
    let lastCount = eventCount;
    const interval = setInterval(() => {
      eventsPerSec = eventCount - lastCount;
      lastCount = eventCount;
    }, 1000);

    return () => {
      clearInterval(interval);
      if (eventSource) {
        eventSource.close();
      }
    };
  });

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    if (e.dataTransfer?.files.length) {
      const file = e.dataTransfer.files[0];
      if (file.size > 5 * 1024 * 1024) {
        uploadWarning = `Warning: Large file detected (${(file.size / 1024 / 1024).toFixed(1)}MB). For best forensic processing speed, please use the Helios CLI. The dashboard will only render the first 1,000 logs to prevent browser freezing.`;
      } else {
        uploadWarning = '';
      }
      uploadFile(file);
    }
  }

  async function uploadFile(file: File) {
    const formData = new FormData();
    formData.append('file', file);
    try {
      const res = await fetch('http://localhost:8080/api/v1/upload', {
        method: 'POST',
        body: formData
      });
      if (res.ok) {
        const data = await res.json();
        events = [...data, ...events].slice(0, 1000);
        eventCount += data.length;
      }
    } catch (err) {
      console.error(err);
    }
  }

  function downloadJSON() {
    const jsonStr = JSON.stringify(events, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'helios_preprocessed_logs.json';
    a.click();
    URL.revokeObjectURL(url);
  }

  let filteredEvents = $derived(
    events.filter((ev) => {
      const matchesSeverity = selectedSeverity === 'ALL' || ev.severity === selectedSeverity;
      const matchesSearch =
        !searchQuery ||
        (ev.message?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false) ||
        (ev.hostname?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false) ||
        (ev.service?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false);
      return matchesSeverity && matchesSearch;
    })
  );

  function severityColor(sev: string | null): string {
    switch (sev) {
      case 'CRIT': return 'text-red-500';
      case 'ERROR': return 'text-red-400';
      case 'WARN': return 'text-amber-400';
      case 'NOTICE': return 'text-blue-400';
      case 'INFO': return 'text-green-400';
      case 'DEBUG': return 'text-gray-400';
      default: return 'text-helios-muted';
    }
  }

  function severityBg(sev: string | null): string {
    switch (sev) {
      case 'CRIT': return 'bg-red-500/10 border-red-500/30';
      case 'ERROR': return 'bg-red-400/10 border-red-400/30';
      case 'WARN': return 'bg-amber-400/10 border-amber-400/30';
      case 'NOTICE': return 'bg-blue-400/10 border-blue-400/30';
      case 'INFO': return 'bg-green-400/10 border-green-400/30';
      case 'DEBUG': return 'bg-gray-400/10 border-gray-400/30';
      default: return 'bg-helios-surface-2 border-helios-border';
    }
  }
</script>


<svelte:head>
  <title>Dashboard — Helios</title>
</svelte:head>


<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="relative flex flex-col h-screen" ondragover={handleDragOver} ondragleave={handleDragLeave} ondrop={handleDrop}>
  <!-- Top bar -->
    <header class="flex items-center justify-between px-6 py-3 border-b border-helios-border bg-helios-surface relative z-10">
    <div class="flex items-center gap-3">
      <a href="/" class="text-xl font-bold text-helios-accent">Helios</a>
      <span class="text-sm text-helios-muted">Dashboard</span>
    </div>
    <div class="flex items-center gap-6 text-sm">
      <button onclick={downloadJSON} class="px-3 py-1.5 text-xs font-semibold rounded bg-helios-surface-2 border border-helios-border hover:bg-helios-border text-helios-text transition-colors">
        Export JSON
      </button>
      <div class="flex items-center gap-2">
        <span class="w-2 h-2 rounded-full {isConnected ? 'bg-helios-green' : 'bg-helios-red'}"></span>
        <span class="text-helios-muted">{isConnected ? 'Connected' : 'Disconnected'}</span>
      </div>
      <div class="text-helios-muted">
        <span class="font-mono text-helios-text">{eventCount}</span> events
      </div>
      <div class="text-helios-muted">
        <span class="font-mono text-helios-text">{eventsPerSec}</span> ev/s
      </div>
      <div class="text-helios-muted">
        <span class="font-mono text-helios-text">{parserCount}</span> parsers
      </div>
    </div>
  </header>

  <div class="flex flex-1 overflow-hidden">
    <!-- Main content -->
    <!-- Dropzone overlay -->
    {#if isDragging}
      <div class="absolute inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm pointer-events-none">
        <div class="p-12 text-center border-4 border-dashed rounded-3xl border-helios-accent bg-helios-surface/80">
          <svg class="w-16 h-16 mx-auto mb-4 text-helios-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
          </svg>
          <h2 class="text-2xl font-bold text-helios-text">Drop log file to analyze</h2>
          <p class="mt-2 text-helios-muted">Supports raw text, syslogs, CEF, JSON, and .evtx binaries</p>
        </div>
      </div>
    {/if}
    
    <main class="flex flex-col flex-1 overflow-hidden">
    {#if uploadWarning}
      <div class="mx-4 mt-4 p-4 text-sm font-semibold border rounded-lg bg-amber-500/10 border-amber-500/30 text-amber-400 flex-shrink-0">
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
      <!-- Charts row -->
      <div class="grid grid-cols-1 gap-4 p-4 lg:grid-cols-2">
        <div class="p-4 border rounded-xl border-helios-border bg-helios-surface">
          <h3 class="mb-3 text-sm font-semibold text-helios-muted">Events Over Time</h3>
          <TimelineChart events={events} />
        </div>
        <div class="p-4 border rounded-xl border-helios-border bg-helios-surface">
          <h3 class="mb-3 text-sm font-semibold text-helios-muted">Severity Distribution</h3>
          <SeverityChart events={events} />
        </div>
      </div>

      <!-- Search bar -->
      <div class="flex items-center gap-3 px-4">
        <div class="relative flex-1">
          <svg xmlns="http://www.w3.org/2000/svg" class="absolute w-4 h-4 -translate-y-1/2 left-3 top-1/2 text-helios-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Search logs by message, hostname, or service..."
            class="w-full py-2.5 pl-10 pr-4 text-sm border rounded-lg bg-helios-surface border-helios-border text-helios-text placeholder:text-helios-muted focus:outline-none focus:border-helios-accent/50"
          />
        </div>
        <select
          bind:value={selectedSeverity}
          class="px-3 py-2.5 text-sm border rounded-lg bg-helios-surface border-helios-border text-helios-text focus:outline-none focus:border-helios-accent/50"
        >
          <option value="ALL">All Severities</option>
          <option value="CRIT">Critical</option>
          <option value="ERROR">Error</option>
          <option value="WARN">Warning</option>
          <option value="NOTICE">Notice</option>
          <option value="INFO">Info</option>
          <option value="DEBUG">Debug</option>
        </select>
      </div>

      <!-- Events table -->
      <div class="flex-1 p-4 overflow-auto">
        <div class="overflow-hidden border rounded-xl border-helios-border">
          <table class="w-full text-sm">
            <thead>
              <tr class="text-left border-b border-helios-border bg-helios-surface-2">
                <th class="px-4 py-3 font-medium text-helios-muted w-44">Timestamp</th>
                <th class="px-4 py-3 font-medium text-helios-muted w-20">Severity</th>
                <th class="px-4 py-3 font-medium text-helios-muted w-40">Host</th>
                <th class="px-4 py-3 font-medium text-helios-muted w-36">Service</th>
                <th class="px-4 py-3 font-medium text-helios-muted">Message</th>
              </tr>
            </thead>
            <tbody>
              {#each filteredEvents as event, i (i)}
                <tr
                  class="border-b cursor-pointer border-helios-border/50 hover:bg-helios-surface-2/50 transition-colors"
                  onclick={() => selectedEvent = selectedEvent === event ? null : event}
                >
                  <td class="px-4 py-2.5 font-mono text-xs text-helios-muted">{new Date(event.timestamp).toLocaleTimeString()}</td>
                  <td class="px-4 py-2.5">
                    <span class="px-2 py-0.5 text-xs font-semibold border rounded {severityBg(event.severity)} {severityColor(event.severity)}">
                      {event.severity ?? '—'}
                    </span>
                  </td>
                  <td class="px-4 py-2.5 font-mono text-xs">{event.hostname ?? '—'}</td>
                  <td class="px-4 py-2.5 font-mono text-xs text-helios-accent">{event.service ?? '—'}</td>
                  <td class="px-4 py-2.5 text-xs truncate max-w-md">{event.message}</td>
                </tr>
                {#if selectedEvent === event}
                  <tr>
                    <td colspan="5" class="p-0">
                      <div class="grid grid-cols-2 gap-0 border-b border-helios-border">
                        <div class="p-4 border-r border-helios-border bg-helios-surface-2/30">
                          <div class="mb-2 text-xs font-semibold text-helios-muted">RAW EVENT</div>
                          <pre class="p-3 overflow-x-auto text-xs rounded-lg bg-helios-bg font-mono text-red-400 whitespace-pre-wrap break-all">{event.raw_event}</pre>
                        </div>
                        <div class="p-4 bg-helios-surface-2/30">
                          <div class="mb-2 text-xs font-semibold text-helios-muted">NORMALIZED EVENT</div>
                          <pre class="p-3 overflow-x-auto text-xs rounded-lg bg-helios-bg font-mono text-green-400 whitespace-pre-wrap">{JSON.stringify({
                            timestamp: event.timestamp,
                            hostname: event.hostname,
                            service: event.service,
                            severity: event.severity,
                            message: event.message
                          }, null, 2)}</pre>
                        </div>
                      </div>
                    </td>
                  </tr>
                {/if}
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    </main>
  </div>
</div>

<script lang="ts">
  import type { ParsedEvent } from '$lib/types';

  let { event, onClose } = $props<{
    event: ParsedEvent | null;
    onClose: () => void;
  }>();

  function formatTime(iso: string) {
    if (!iso) return '—';
    return new Date(iso).toLocaleString();
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

{#if event}
<div class="w-96 border-l border-helios-border bg-helios-bg flex flex-col h-full overflow-hidden shadow-2xl z-20 shrink-0">
  <!-- Header -->
  <div class="flex items-center justify-between p-4 border-b border-helios-border bg-helios-surface">
    <h3 class="font-semibold text-helios-text">Log Inspector</h3>
    <button onclick={onClose} class="text-helios-muted hover:text-helios-text transition-colors" aria-label="Close Inspector">
      <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>

  <div class="flex-1 overflow-y-auto p-4 space-y-6">
    <!-- Meta fields -->
    <div class="space-y-3">
      <div>
        <div class="text-xs font-semibold text-helios-muted mb-1">TIMESTAMP</div>
        <div class="font-mono text-sm text-helios-text">{formatTime(event.timestamp)}</div>
      </div>
      <div>
        <div class="text-xs font-semibold text-helios-muted mb-1">SEVERITY</div>
        <span class="inline-block px-2 py-0.5 text-xs font-semibold border rounded {severityBg(event.severity)}">
          {event.severity ?? '—'}
        </span>
      </div>
      <div>
        <div class="text-xs font-semibold text-helios-muted mb-1">HOSTNAME</div>
        <div class="font-mono text-sm text-helios-text">{event.hostname ?? '—'}</div>
      </div>
      <div>
        <div class="text-xs font-semibold text-helios-muted mb-1">SERVICE</div>
        <div class="font-mono text-sm text-helios-accent">{event.service ?? '—'}</div>
      </div>
    </div>

    <!-- Message -->
    <div>
      <div class="text-xs font-semibold text-helios-muted mb-2">NORMALIZED MESSAGE</div>
      <div class="p-3 text-sm rounded-lg bg-helios-surface-2 border border-helios-border text-helios-text whitespace-pre-wrap break-all">
        {event.message}
      </div>
    </div>

    <!-- Raw Event -->
    <div>
      <div class="text-xs font-semibold text-helios-muted mb-2">RAW EVENT</div>
      <pre class="p-3 overflow-x-auto text-xs rounded-lg bg-[#11111a] border border-helios-border font-mono text-helios-muted whitespace-pre-wrap break-all">{event.raw_event}</pre>
    </div>
    
    <!-- JSON Normalized -->
    <div>
      <div class="text-xs font-semibold text-helios-muted mb-2">JSON NORMALIZED</div>
      <pre class="p-3 overflow-x-auto text-xs rounded-lg bg-[#11111a] border border-helios-border font-mono text-green-400 whitespace-pre-wrap">{JSON.stringify({
        timestamp: event.timestamp,
        hostname: event.hostname,
        service: event.service,
        severity: event.severity,
        message: event.message
      }, null, 2)}</pre>
    </div>
  </div>
</div>
{/if}

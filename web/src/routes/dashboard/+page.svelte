<script lang="ts">
  import { onMount } from 'svelte';
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
  let parserCount = $state(4);
  let selectedEvent = $state<ParsedEvent | null>(null);

  // Mock data for demo — replace with WebSocket / API calls later
  const MOCK_EVENTS: ParsedEvent[] = [
    {
      timestamp: '2026-08-26T22:14:15Z',
      hostname: 'firewall1.localdomain',
      service: '%ASA-4-106023',
      severity: 'WARN',
      message: 'Deny tcp src outside:192.168.1.1/1234 dst inside:10.0.0.1/80 by access-group "outside_in"',
      raw_event: '<34>Aug 26 22:14:15 firewall1.localdomain %ASA-4-106023: Deny tcp src outside:192.168.1.1/1234 dst inside:10.0.0.1/80 by access-group "outside_in"'
    },
    {
      timestamp: '2026-08-26T22:14:16Z',
      hostname: 'webserver.prod',
      service: 'nginx',
      severity: 'INFO',
      message: 'GET /api/health 200 0.003s',
      raw_event: '{"timestamp":"2026-08-26T22:14:16Z","level":"info","service":"nginx","message":"GET /api/health 200 0.003s"}'
    },
    {
      timestamp: '2026-08-26T22:14:17Z',
      hostname: 'db-primary',
      service: 'postgresql',
      severity: 'ERROR',
      message: 'connection limit exceeded for non-superuser connections',
      raw_event: '<11>Aug 26 22:14:17 db-primary postgresql[4521]: FATAL: connection limit exceeded for non-superuser connections'
    },
    {
      timestamp: '2026-08-26T22:14:18Z',
      hostname: 'k8s-node-02',
      service: 'kubelet',
      severity: 'CRIT',
      message: 'NodeNotReady condition detected, pod eviction initiated',
      raw_event: '{"timestamp":"2026-08-26T22:14:18Z","level":"critical","source":"kubelet","host":"k8s-node-02","message":"NodeNotReady condition detected, pod eviction initiated"}'
    },
    {
      timestamp: '2026-08-26T22:14:19Z',
      hostname: 'switch-core-01',
      service: 'sshd',
      severity: 'INFO',
      message: 'Accepted publickey for admin from 10.0.0.5 port 52341 ssh2',
      raw_event: '<86>Aug 26 22:14:19 switch-core-01 sshd[9821]: Accepted publickey for admin from 10.0.0.5 port 52341 ssh2'
    },
    {
      timestamp: '2026-08-26T22:14:20Z',
      hostname: 'paloalto-fw',
      service: '%PAN-6-THREAT',
      severity: 'WARN',
      message: 'Spyware detected: Trojan.GenericKD source=203.0.113.15 dest=10.1.2.50 action=drop',
      raw_event: '<36>Aug 26 22:14:20 paloalto-fw %PAN-6-THREAT: Spyware detected: Trojan.GenericKD source=203.0.113.15 dest=10.1.2.50 action=drop'
    },
    {
      timestamp: '2026-08-26T22:14:21Z',
      hostname: 'fortinet-edge',
      service: '%FORTINET-5-0100',
      severity: 'NOTICE',
      message: 'SSL VPN tunnel established user=john.doe src=198.51.100.22 duration=0s',
      raw_event: '<45>Aug 26 22:14:21 fortinet-edge %FORTINET-5-0100: SSL VPN tunnel established user=john.doe src=198.51.100.22 duration=0s'
    },
    {
      timestamp: '2026-08-26T22:14:22Z',
      hostname: 'app-server-03',
      service: 'helios',
      severity: 'DEBUG',
      message: 'Parser registry initialized with 11 plugins: [syslog, json, cef, apache, nginx, android, openssh, proxifier, spark, windows, zookeeper]',
      raw_event: '{"timestamp":"2026-08-26T22:14:22Z","level":"debug","service":"helios","message":"Parser registry initialized with 11 plugins: [syslog, json, cef, apache, nginx, android, openssh, proxifier, spark, windows, zookeeper]"}'
    }
  ];

  onMount(() => {
    isConnected = true;
    eventCount = MOCK_EVENTS.length;
    eventsPerSec = 142;
    // Simulate events arriving over time
    MOCK_EVENTS.forEach((ev, i) => {
      setTimeout(() => {
        events = [...events, ev];
        eventCount = events.length;
      }, i * 300);
    });
  });

  let filteredEvents = $derived(
    events.filter((ev) => {
      const matchesSeverity = selectedSeverity === 'ALL' || ev.severity === selectedSeverity;
      const matchesSearch =
        !searchQuery ||
        ev.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
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

<div class="flex flex-col h-screen">
  <!-- Top bar -->
  <header class="flex items-center justify-between px-6 py-3 border-b border-helios-border bg-helios-surface">
    <div class="flex items-center gap-3">
      <a href="/" class="text-xl font-bold text-helios-accent">Helios</a>
      <span class="text-sm text-helios-muted">Dashboard</span>
    </div>
    <div class="flex items-center gap-6 text-sm">
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
    <main class="flex flex-col flex-1 overflow-hidden">
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
              {#each filteredEvents as event (event.timestamp + event.message)}
                <tr
                  class="border-b cursor-pointer border-helios-border/50 hover:bg-helios-surface-2/50 transition-colors"
                  onclick={() => selectedEvent = selectedEvent === event ? null : event}
                >
                  <td class="px-4 py-2.5 font-mono text-xs text-helios-muted">{new Date(event.timestamp).toLocaleTimeString()}</td>
                  <td class="px-4 py-2.5">
                    <span class="px-2 py-0.5 text-xs font-semibold border rounded {severityBg(event.severity)} {severityColor(event.severity)}">
                      {event.severity ?? 'UNKNOWN'}
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

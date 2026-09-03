<script lang="ts">
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';

  interface Event {
    severity: string | null;
  }

  let { events, onSeveritySelect }: { events: Event[], onSeveritySelect?: (sev: string) => void } = $props();
  let chartEl: HTMLDivElement;
  let chart: echarts.ECharts;

  const SEVERITY_COLORS: Record<string, string> = {
    CRIT: '#ef4444',
    ERROR: '#f87171',
    WARN: '#f59e0b',
    NOTICE: '#3b82f6',
    INFO: '#22c55e',
    DEBUG: '#6b7280'
  };

  function updateChart(events: Event[]) {
    if (!chart) return;

    const counts: Record<string, number> = {};
    for (const ev of events) {
      const sev = ev.severity || 'Unknown';
      counts[sev] = (counts[sev] || 0) + 1;
    }

    const data = Object.entries(counts).map(([name, value]) => ({
      name,
      value,
      itemStyle: { color: SEVERITY_COLORS[name] ?? '#888' }
    }));

    chart.setOption({
      tooltip: { trigger: 'item', backgroundColor: '#1a1a26', borderColor: '#2a2a3a', textStyle: { color: '#e4e4ef' } },
      series: [
        {
          type: 'pie',
          radius: ['45%', '70%'],
          avoidLabelOverlap: false,
          label: { show: true, color: '#8888a0', fontSize: 11 },
          emphasis: { label: { show: true, fontSize: 14, fontWeight: 'bold' } },
          data
        }
      ]
    });
  }

  onMount(() => {
    chart = echarts.init(chartEl, undefined, { renderer: 'canvas' });
    
    chart.on('click', (params: any) => {
      if (onSeveritySelect && params.name) {
        onSeveritySelect(params.name);
      }
    });

    updateChart(events);

    const observer = new ResizeObserver(() => chart?.resize());
    observer.observe(chartEl);

    return () => {
      observer.disconnect();
      chart?.dispose();
    };
  });

  $effect(() => {
    events;
    updateChart(events);
  });
</script>

<div bind:this={chartEl} class="w-full h-48"></div>

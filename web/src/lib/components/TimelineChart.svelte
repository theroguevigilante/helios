<script lang="ts">
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';

  interface Event {
    timestamp: string;
  }

  let { events }: { events: Event[] } = $props();
  let chartEl: HTMLDivElement;
  let chart: echarts.ECharts;

  function updateChart(events: Event[]) {
    if (!chart) return;

    // Bucket events by second
    const buckets: Record<string, number> = {};
    for (const ev of events) {
      const sec = ev.timestamp.slice(0, 19);
      buckets[sec] = (buckets[sec] || 0) + 1;
    }

    const sorted = Object.entries(buckets).sort(([a], [b]) => a.localeCompare(b));

    chart.setOption({
      tooltip: { trigger: 'axis', backgroundColor: '#1a1a26', borderColor: '#2a2a3a', textStyle: { color: '#e4e4ef' } },
      grid: { top: 10, right: 16, bottom: 24, left: 40 },
      xAxis: {
        type: 'category',
        data: sorted.map(([t]) => t.slice(11)),
        axisLabel: { color: '#8888a0', fontSize: 10 },
        axisLine: { lineStyle: { color: '#2a2a3a' } }
      },
      yAxis: {
        type: 'value',
        splitLine: { lineStyle: { color: '#2a2a3a' } },
        axisLabel: { color: '#8888a0', fontSize: 10 }
      },
      series: [
        {
          type: 'bar',
          data: sorted.map(([, v]) => v),
          itemStyle: { color: '#f59e0b', borderRadius: [4, 4, 0, 0] },
          barWidth: '60%'
        }
      ]
    });
  }

  onMount(() => {
    chart = echarts.init(chartEl, undefined, { renderer: 'canvas' });
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

<script lang="ts">
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';

  interface Event {
    timestamp: string;
  }

  let { events, displayTimezone = 'Local', timeRange = null, onTimeRangeSelect }: { events: Event[], displayTimezone?: 'Local' | 'UTC', timeRange?: [string, string] | null, onTimeRangeSelect?: (range: [string, string] | null) => void } = $props();
  let chartEl: HTMLDivElement;
  let chart: echarts.ECharts;
  let fullTimestamps: string[] = []; // Keep the original ISO strings

  function updateChart(events: Event[], tz: 'Local' | 'UTC') {
    if (!chart) return;

    // Bucket events by second
    const buckets: Record<string, number> = {};
    for (const ev of events) {
      const sec = ev.timestamp.slice(0, 19);
      buckets[sec] = (buckets[sec] || 0) + 1;
    }

    const sorted = Object.entries(buckets).sort(([a], [b]) => a.localeCompare(b));
    fullTimestamps = sorted.map(([t]) => t); // The full YYYY-MM-DDTHH:mm:ss

    chart.setOption({
      tooltip: { trigger: 'axis', backgroundColor: '#1a1a26', borderColor: '#2a2a3a', textStyle: { color: '#e4e4ef' } },
      grid: { top: 40, right: 16, bottom: 40, left: 40 },
            toolbox: {
        feature: {
          brush: { type: ['lineX', 'clear'] }
        },
        top: -5,
        right: 10,
        iconStyle: { borderColor: '#8888a0' }
      },
      brush: {
        
        xAxisIndex: 'all',
        outOfBrush: { colorAlpha: 0.1 }
      },
      xAxis: {
        type: 'category',
        data: sorted.map(([t]) => tz === 'UTC' ? new Date(t + 'Z').toLocaleTimeString('en-US', { timeZone: 'UTC', hour12: false }) : new Date(t + 'Z').toLocaleTimeString()),
        axisLabel: { color: '#8888a0', fontSize: 10, hideOverlap: true },
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
          itemStyle: { 
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#38bdf8' }, // sky-400
              { offset: 1, color: '#818cf8' }  // indigo-400
            ]),
            borderRadius: [4, 4, 0, 0] 
          },
          barWidth: '60%'
        }
      ]
    });
  }

  onMount(() => {
    chart = echarts.init(chartEl, undefined, { renderer: 'canvas' });
    
    chart.on('click', (params: any) => {
      if (params.componentType === 'series') {
        const t = fullTimestamps[params.dataIndex];
        if (t) {
          if (timeRange && timeRange[0] === t && timeRange[1] === t) {
            if (onTimeRangeSelect) onTimeRangeSelect(null);
          } else {
            if (onTimeRangeSelect) onTimeRangeSelect([t, t]);
          }
        }
      }
    });

    chart.on('brushEnd', (params: any) => {
      if (!params.areas || params.areas.length === 0) {
        if (onTimeRangeSelect) onTimeRangeSelect(null);
        return;
      }
      
      const area = params.areas[0];
      const range = area.coordRange;
      
      if (!range || range.length < 2) return;
      
      const startIndex = Math.max(0, Math.floor(range[0]));
      const endIndex = Math.min(fullTimestamps.length - 1, Math.ceil(range[1]));
      
      if (onTimeRangeSelect && fullTimestamps[startIndex] && fullTimestamps[endIndex]) {
        onTimeRangeSelect([fullTimestamps[startIndex], fullTimestamps[endIndex]]);
      }
    });

    updateChart(events, displayTimezone);

    const observer = new ResizeObserver(() => chart?.resize());
    observer.observe(chartEl);

    return () => {
      observer.disconnect();
      chart?.dispose();
    };
  });

  $effect(() => {
    events;
    updateChart(events, displayTimezone);
  });
</script>

<div bind:this={chartEl} class="w-full h-full min-h-[160px]"></div>

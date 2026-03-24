<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { X } from 'lucide-svelte';
  import { Chart, registerables } from 'chart.js';

  Chart.register(...registerables);

  export let visible = false;

  interface ProcessMetrics {
    cpu_percent: number;
    memory_mb: number;
    pid: number;
  }

  interface SystemMetrics {
    total_cpu_percent: number;
    total_memory_mb: number;
    used_memory_mb: number;
    cpu_count: number;
  }

  interface ResourceMetrics {
    timestamp: number;
    main_process: ProcessMetrics;
    extension_host: ProcessMetrics | null;
    system: SystemMetrics;
  }

  let cpuChart: Chart | null = null;
  let memoryChart: Chart | null = null;
  let cpuCanvas: HTMLCanvasElement;
  let memoryCanvas: HTMLCanvasElement;

  let metrics: ResourceMetrics | null = null;
  let updateInterval: number;

  const maxDataPoints = 60; // Keep last 60 seconds
  const cpuData = {
    labels: [] as string[],
    mainProcess: [] as number[],
    extHost: [] as number[],
    system: [] as number[],
  };
  const memoryData = {
    labels: [] as string[],
    mainProcess: [] as number[],
    extHost: [] as number[],
    systemUsed: [] as number[],
  };

  async function fetchMetrics() {
    try {
      console.log('Fetching metrics...');
      metrics = await invoke<ResourceMetrics>('get_resource_metrics');
      console.log('Received metrics:', metrics);
      updateCharts();
    } catch (error) {
      console.error('Failed to fetch metrics:', error);
    }
  }

  function updateCharts() {
    if (!metrics) return;

    const timestamp = new Date(metrics.timestamp).toLocaleTimeString();

    // Add new data
    cpuData.labels.push(timestamp);
    cpuData.mainProcess.push(metrics.main_process.cpu_percent);
    cpuData.extHost.push(metrics.extension_host?.cpu_percent || 0);
    cpuData.system.push(metrics.system.total_cpu_percent);

    memoryData.labels.push(timestamp);
    memoryData.mainProcess.push(metrics.main_process.memory_mb);
    memoryData.extHost.push(metrics.extension_host?.memory_mb || 0);
    memoryData.systemUsed.push(metrics.system.used_memory_mb);

    // Keep only last maxDataPoints
    if (cpuData.labels.length > maxDataPoints) {
      cpuData.labels.shift();
      cpuData.mainProcess.shift();
      cpuData.extHost.shift();
      cpuData.system.shift();

      memoryData.labels.shift();
      memoryData.mainProcess.shift();
      memoryData.extHost.shift();
      memoryData.systemUsed.shift();
    }

    // Update charts
    if (cpuChart) {
      cpuChart.data.labels = cpuData.labels;
      cpuChart.data.datasets[0].data = cpuData.mainProcess;
      cpuChart.data.datasets[1].data = cpuData.extHost;
      cpuChart.data.datasets[2].data = cpuData.system;
      cpuChart.update('none');
    }

    if (memoryChart) {
      memoryChart.data.labels = memoryData.labels;
      memoryChart.data.datasets[0].data = memoryData.mainProcess;
      memoryChart.data.datasets[1].data = memoryData.extHost;
      memoryChart.data.datasets[2].data = memoryData.systemUsed;
      memoryChart.update('none');
    }
  }

  function initCharts() {
    if (cpuCanvas && !cpuChart) {
      cpuChart = new Chart(cpuCanvas, {
        type: 'line',
        data: {
          labels: [],
          datasets: [
            {
              label: 'Main Process CPU %',
              data: [],
              borderColor: '#3b82f6',
              backgroundColor: 'rgba(59, 130, 246, 0.1)',
              tension: 0.4,
            },
            {
              label: 'Extension Host CPU %',
              data: [],
              borderColor: '#10b981',
              backgroundColor: 'rgba(16, 185, 129, 0.1)',
              tension: 0.4,
            },
            {
              label: 'System CPU %',
              data: [],
              borderColor: '#f59e0b',
              backgroundColor: 'rgba(245, 158, 11, 0.1)',
              tension: 0.4,
            },
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          animation: false,
          scales: {
            y: {
              beginAtZero: true,
              max: 100,
              ticks: { color: '#9ca3af' },
              grid: { color: '#374151' },
            },
            x: {
              ticks: { color: '#9ca3af', maxRotation: 0 },
              grid: { color: '#374151' },
            },
          },
          plugins: {
            legend: {
              labels: { color: '#d1d5db' },
            },
          },
        },
      });
    }

    if (memoryCanvas && !memoryChart) {
      memoryChart = new Chart(memoryCanvas, {
        type: 'line',
        data: {
          labels: [],
          datasets: [
            {
              label: 'Main Process Memory (MB)',
              data: [],
              borderColor: '#3b82f6',
              backgroundColor: 'rgba(59, 130, 246, 0.1)',
              tension: 0.4,
            },
            {
              label: 'Extension Host Memory (MB)',
              data: [],
              borderColor: '#10b981',
              backgroundColor: 'rgba(16, 185, 129, 0.1)',
              tension: 0.4,
            },
            {
              label: 'System Used Memory (MB)',
              data: [],
              borderColor: '#f59e0b',
              backgroundColor: 'rgba(245, 158, 11, 0.1)',
              tension: 0.4,
            },
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          animation: false,
          scales: {
            y: {
              beginAtZero: true,
              ticks: { color: '#9ca3af' },
              grid: { color: '#374151' },
            },
            x: {
              ticks: { color: '#9ca3af', maxRotation: 0 },
              grid: { color: '#374151' },
            },
          },
          plugins: {
            legend: {
              labels: { color: '#d1d5db' },
            },
          },
        },
      });
    }
  }

  function close() {
    visible = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  onMount(() => {
    if (visible) {
      initCharts();
      fetchMetrics();
      updateInterval = window.setInterval(fetchMetrics, 1000);
    }
  });

  onDestroy(() => {
    if (updateInterval) {
      clearInterval(updateInterval);
    }
    if (cpuChart) {
      cpuChart.destroy();
      cpuChart = null;
    }
    if (memoryChart) {
      memoryChart.destroy();
      memoryChart = null;
    }
  });

  $: if (visible && cpuCanvas && memoryCanvas) {
    initCharts();
    fetchMetrics();
    if (!updateInterval) {
      updateInterval = window.setInterval(fetchMetrics, 1000);
    }
  } else if (!visible && updateInterval) {
    clearInterval(updateInterval);
    updateInterval = 0;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible}
  <div class="metrics-overlay" on:click={handleOverlayClick} role="presentation" tabindex="-1">
    <div class="metrics-modal" role="dialog" aria-modal="true" aria-label="Resource metrics">
      <div class="metrics-header">
        <h2>Resource Metrics</h2>
        <button class="close-btn" on:click={close}>
          <X size={20} />
        </button>
      </div>

      <div class="metrics-content">
        {#if metrics}
          <div class="metrics-summary">
            <div class="metric-card">
              <h3>Main Process</h3>
              <p>CPU: {metrics.main_process.cpu_percent.toFixed(1)}%</p>
              <p>Memory: {metrics.main_process.memory_mb.toFixed(1)} MB</p>
              <p class="pid">PID: {metrics.main_process.pid}</p>
            </div>

            {#if metrics.extension_host}
              <div class="metric-card">
                <h3>Extension Host</h3>
                <p>CPU: {metrics.extension_host.cpu_percent.toFixed(1)}%</p>
                <p>Memory: {metrics.extension_host.memory_mb.toFixed(1)} MB</p>
                <p class="pid">PID: {metrics.extension_host.pid}</p>
              </div>
            {/if}

            <div class="metric-card">
              <h3>System</h3>
              <p>CPU: {metrics.system.total_cpu_percent.toFixed(1)}%</p>
              <p>
                Memory: {metrics.system.used_memory_mb.toFixed(0)} /
                {metrics.system.total_memory_mb.toFixed(0)} MB
              </p>
              <p class="pid">{metrics.system.cpu_count} CPUs</p>
            </div>
          </div>

          <div class="charts">
            <div class="chart-container">
              <h3>CPU Usage</h3>
              <canvas bind:this={cpuCanvas}></canvas>
            </div>

            <div class="chart-container">
              <h3>Memory Usage</h3>
              <canvas bind:this={memoryCanvas}></canvas>
            </div>
          </div>
        {:else}
          <p class="loading">Loading metrics...</p>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .metrics-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
  }

  .metrics-modal {
    background: #1f2937;
    border-radius: 8px;
    width: 90%;
    max-width: 1200px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.3);
  }

  .metrics-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #374151;
  }

  .metrics-header h2 {
    margin: 0;
    font-size: 18px;
    color: #f3f4f6;
  }

  .close-btn {
    background: none;
    border: none;
    color: #9ca3af;
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: #374151;
    color: #f3f4f6;
  }

  .metrics-content {
    padding: 20px;
    overflow-y: auto;
  }

  .metrics-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
  }

  .metric-card {
    background: #111827;
    border-radius: 6px;
    padding: 16px;
  }

  .metric-card h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: #9ca3af;
    font-weight: 500;
  }

  .metric-card p {
    margin: 6px 0;
    color: #f3f4f6;
    font-size: 14px;
  }

  .metric-card .pid {
    color: #6b7280;
    font-size: 12px;
  }

  .charts {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }

  .chart-container {
    background: #111827;
    border-radius: 6px;
    padding: 16px;
  }

  .chart-container h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: #9ca3af;
    font-weight: 500;
  }

  .chart-container canvas {
    height: 250px !important;
  }

  .loading {
    text-align: center;
    color: #9ca3af;
    padding: 40px;
  }

  @media (max-width: 900px) {
    .charts {
      grid-template-columns: 1fr;
    }
  }
</style>

<script lang="ts">
  import '../app.css';
  import { initTheme } from '$lib/theme.svelte';
  
  initTheme();
  
  let { children } = $props();
</script>

<div class="min-h-screen bg-helios-bg text-helios-text">
  {@render children()}
</div>

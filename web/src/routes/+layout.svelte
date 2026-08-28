<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

<div class="min-h-screen bg-helios-bg text-helios-text">
  {@render children()}
</div>

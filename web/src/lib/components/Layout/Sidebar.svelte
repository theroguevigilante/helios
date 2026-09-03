<script lang="ts">
  let { searchQuery = $bindable(), selectedSeverity = $bindable() } = $props<{
    searchQuery: string;
    selectedSeverity: string;
  }>();

</script>

<aside class="w-64 flex flex-col border-r border-helios-border bg-helios-bg h-full shrink-0">
  <div class="p-4 border-b border-helios-border bg-helios-surface flex items-center justify-between">
    <h3 class="font-semibold text-helios-text text-sm">Forensic Filters</h3>
  </div>

  <div class="p-4 space-y-6 overflow-y-auto flex-1">
    
    <!-- Search Query -->
    <div class="space-y-2">
      <label class="text-xs font-semibold text-helios-muted" for="search">GLOBAL SEARCH</label>
      <div class="relative">
        <svg xmlns="http://www.w3.org/2000/svg" class="absolute w-4 h-4 -translate-y-1/2 left-3 top-1/2 text-helios-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          id="search"
          type="text"
          bind:value={searchQuery}
          placeholder="Regex or raw string..."
          class="w-full py-2 pl-9 pr-3 text-sm border rounded bg-helios-surface-2 border-helios-border text-helios-text placeholder:text-helios-muted/50 focus:outline-none focus:border-helios-accent/50"
        />
      </div>
    </div>

    <!-- Severity Filter -->
    <div class="space-y-2">
      <label class="text-xs font-semibold text-helios-muted" for="severity">SEVERITY LEVEL</label>
      <select
        id="severity"
        bind:value={selectedSeverity}
        class="w-full px-3 py-2 text-sm border rounded bg-helios-surface-2 border-helios-border text-helios-text focus:outline-none focus:border-helios-accent/50"
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

    <!-- More filters can be added here later (Time range, Host, Service facets) -->
    
  </div>
</aside>

<script lang="ts">
  let { searchQuery = $bindable(), selectedSeverity = $bindable(), onFileUpload } = $props<{
    searchQuery: string;
    selectedSeverity: string;
    onFileUpload: (file: File) => void;
  }>();
  
  let isDragging = $state(false);

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
    if (e.dataTransfer?.files?.length) {
      onFileUpload(e.dataTransfer.files[0]);
    }
  }

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

    <!-- Drag & Drop Zone -->
    <div class="space-y-2 mt-6">
      <label for="file-upload" class="text-xs font-semibold text-helios-muted">UPLOAD LOG FILE</label>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div 
        class="border-2 border-dashed rounded-xl p-6 text-center transition-colors cursor-pointer {isDragging ? 'border-helios-accent bg-helios-accent/10' : 'border-helios-border bg-helios-surface-2 hover:border-helios-accent/50'}"
        ondragover={handleDragOver}
        ondragleave={handleDragLeave}
        ondrop={handleDrop}
        onclick={() => document.getElementById('file-upload')?.click()}
      >
        <svg class="w-8 h-8 mx-auto mb-2 transition-colors {isDragging ? 'text-helios-accent' : 'text-helios-muted'}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
        <div class="text-xs font-semibold transition-colors {isDragging ? 'text-helios-text' : 'text-helios-muted'}">
          {isDragging ? 'Drop file here' : 'Drag & drop or click'}
        </div>
        <div class="text-[10px] text-helios-muted/70 mt-1">
          .log, .json, .evtx
        </div>
        <input type="file" id="file-upload" class="hidden" onchange={(e) => {
          const files = (e.target as HTMLInputElement).files;
          if (files && files.length > 0) onFileUpload(files[0]);
        }} />
      </div>
    </div>

    
  </div>
</aside>

<script lang="ts">
  let { searchQuery = $bindable(), selectedSeverity = $bindable(), onFileUpload } = $props<{
    searchQuery: string;
    selectedSeverity: string;
    onFileUpload: (file: File) => void;
  }>();
  
  let isDragging = $state(false);
  let showParsersModal = $state(false);
  let parsers = $state<{name: string, type: string, description: string}[]>([]);
  let isLoadingParsers = $state(false);

  async function loadParsers() {
    showParsersModal = true;
    isLoadingParsers = true;
    try {
      const res = await fetch('/api/v1/parsers');
      if (res.ok) {
        parsers = await res.json();
      }
    } catch (e) {
      console.error("Failed to load parsers", e);
    } finally {
      isLoadingParsers = false;
    }
  }


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
        <option value="—">—</option>
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

    

    <!-- Active Parsers Button -->
    <div class="mt-auto pt-6 border-t border-helios-border">
      <button 
        onclick={loadParsers}
        class="w-full flex items-center justify-center gap-2 px-4 py-2 text-sm font-semibold text-helios-muted bg-helios-surface-2 border border-helios-border rounded hover:text-helios-text hover:border-helios-accent/50 transition-colors"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
        </svg>
        View Loaded Parsers
      </button>
    </div>
  </div>
</aside>


<!-- Parsers Modal -->
{#if showParsersModal}
  <div role="button" tabindex="0" aria-label="Close modal backdrop" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onclick={() => showParsersModal = false} onkeydown={(e) => { if (e.key === 'Escape' || e.key === 'Enter') showParsersModal = false; }}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div role="dialog" tabindex="-1" class="bg-helios-surface border border-helios-border rounded-xl shadow-2xl w-full max-w-2xl flex flex-col max-h-[80vh]" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <div class="p-4 border-b border-helios-border flex justify-between items-center bg-helios-surface-2 rounded-t-xl">
        <h2 class="text-lg font-bold text-helios-text flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-helios-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
          </svg>
          Active Parsers ({parsers.length})
        </h2>
        <button aria-label="Close modal" class="text-helios-muted hover:text-helios-text" onclick={() => showParsersModal = false}>
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      
      <div class="p-0 overflow-y-auto">
        {#if isLoadingParsers}
          <div class="p-8 text-center text-helios-muted">Loading parsers...</div>
        {:else if parsers.length === 0}
          <div class="p-8 text-center text-helios-muted">No parsers loaded.</div>
        {:else}
          <table class="w-full text-left border-collapse">
            <thead class="bg-helios-surface-2 sticky top-0 shadow-sm">
              <tr>
                <th class="p-3 text-xs font-semibold text-helios-muted border-b border-helios-border">NAME</th>
                <th class="p-3 text-xs font-semibold text-helios-muted border-b border-helios-border">TYPE</th>
                <th class="p-3 text-xs font-semibold text-helios-muted border-b border-helios-border">DESCRIPTION</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-helios-border">
              {#each parsers as parser}
                <tr class="hover:bg-helios-surface-2/50 transition-colors">
                  <td class="p-3 text-sm font-medium text-helios-text">{parser.name}</td>
                  <td class="p-3">
                    {#if parser.type === 'lisp'}
                      <div class="flex items-center gap-1">
                        <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-bold bg-[#7c3aed]/20 text-[#c4b5fd] border border-[#7c3aed]/50">
                          LISP DSL
                        </span>
                        <div class="relative group flex items-center">
                          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 text-helios-muted hover:text-[#c4b5fd] cursor-help transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                          </svg>
                          <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 hidden w-64 p-3 text-[11px] leading-relaxed text-helios-text bg-helios-surface border border-helios-border rounded-lg shadow-xl group-hover:block z-[60] normal-case font-normal tracking-normal text-left">
                            To extend Helios, a forensic expert can write a Scheme script or ask the AI Assistant to generate one on the fly for custom log formats. However, for maximum native throughput on high-volume logs, we recommend extending the engine using Rust.
                          </div>
                        </div>
                      </div>
                    {:else}
                      <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-bold bg-helios-border/50 text-helios-muted border border-helios-border">
                        NATIVE
                      </span>
                    {/if}
                  </td>
                  <td class="p-3 text-sm text-helios-muted">{parser.description}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    </div>
  </div>
{/if}

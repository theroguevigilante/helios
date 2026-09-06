<script lang="ts">
  import { onMount } from 'svelte';
  import type { ParsedEvent } from '$lib/types';
  import { chatHistoryStore, type ChatMessage } from '$lib/ai';
  import { marked } from 'marked';

  let { event, onClose, displayTimezone = 'Local' } = $props<{
    event: ParsedEvent | null;
    onClose: () => void;
    displayTimezone?: 'Local' | 'UTC';
  }>();

  function formatTime(iso: string) {
    if (!iso) return '—';
    const d = new Date(iso);
    return displayTimezone === 'UTC' 
      ? d.toLocaleString('en-US', { timeZone: 'UTC', hour12: false }) + ' UTC'
      : d.toLocaleString();
  }

  function severityBg(sev: string | null): string {
    switch (sev) {
      case 'CRIT': return 'bg-red-500/10 border-red-500/30 text-red-400';
      case 'ERROR': return 'bg-red-400/10 border-red-400/30 text-red-400';
      case 'WARN': return 'bg-amber-400/10 border-amber-400/30 text-amber-400';
      case 'NOTICE': return 'bg-blue-400/10 border-blue-400/30 text-blue-400';
      case 'INFO': return 'bg-green-400/10 border-green-400/30 text-green-400';
      case 'DEBUG': return 'bg-gray-400/10 border-gray-400/30 text-gray-400';
      default: return 'bg-helios-surface-2 border-helios-border text-helios-muted';
    }
  }

  let aiMode = $state<'idle' | 'warning' | 'chat'>('idle');
  let provider = $state('gemini');
  let modelName = $state('gemini-1.5-flash');
  let apiKey = $state('');
  let chatInput = $state('');
  let isGenerating = $state(false);

  // Subscribe to global chat history
  let chatHistory = $state<ChatMessage[]>([]);
  chatHistoryStore.subscribe(v => chatHistory = v);

  onMount(() => {
    provider = localStorage.getItem('helios_ai_provider') || 'gemini';
    apiKey = localStorage.getItem(`helios_ai_key_${provider}`) || '';
  });

  function saveSettings() {
    localStorage.setItem('helios_ai_provider', provider);
    if (provider !== 'ollama') {
        localStorage.setItem(`helios_ai_key_${provider}`, apiKey);
    }
  }

  function switchProvider(p: string) {
    provider = p;
    apiKey = localStorage.getItem(`helios_ai_key_${p}`) || '';
    if (p === 'gemini') modelName = 'gemini-1.5-flash';
    else if (p === 'openai') modelName = 'gpt-3.5-turbo';
    else if (p === 'groq') modelName = 'llama3-8b-8192';
    else if (p === 'openrouter') modelName = 'meta-llama/llama-3.1-8b-instruct:free';
    else if (p === 'ollama') modelName = 'llama3';
    saveSettings();
  }

  function openAI() {
    if (localStorage.getItem('helios_ai_warning_accepted')) {
        aiMode = 'chat';
    } else {
        aiMode = 'warning';
    }
  }

  function acceptWarning() {
    localStorage.setItem('helios_ai_warning_accepted', 'true');
    aiMode = 'chat';
  }

  async function sendMessage() {
    if (!chatInput.trim() || isGenerating) return;
    if (provider !== 'ollama' && !apiKey) return;

    const text = chatInput.trim();
    chatHistoryStore.update(h => [...h, { role: 'user', text }]);
    chatInput = '';
    isGenerating = true;

    try {
        const prompt = `You are Helios AI, an expert cybersecurity and systems analyst. Analyze the following log event and answer the user's query.

LOG EVENT:
${JSON.stringify(event, null, 2)}

USER QUERY: ${text}`;
        let reply = "";

        const messages = chatHistory.map(h => ({
            role: h.role === 'model' ? 'assistant' : 'user',
            content: h.text
        }));
        messages.push({ role: 'user', content: prompt });

        if (provider === 'gemini') {
            const res = await fetch(`https://generativelanguage.googleapis.com/v1beta/models/${modelName}:generateContent?key=${apiKey}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ contents: [{ parts: [{ text: prompt }] }] })
            });
            if (!res.ok) {
                const errData = await res.json().catch(() => ({}));
                throw new Error(errData.error?.message || `HTTP ${res.status} API Error`);
            }
            const data = await res.json();
            if (!data.candidates || !data.candidates[0].content) {
                throw new Error("Safety blocked or invalid response from Gemini API.");
            }
            reply = data.candidates[0].content.parts[0].text;

        } else if (provider === 'openai' || provider === 'groq' || provider === 'openrouter') {
            let endpoint = 'https://api.openai.com/v1/chat/completions';
            let model = modelName;
            
            if (provider === 'groq') {
                endpoint = 'https://api.groq.com/openai/v1/chat/completions';
            } else if (provider === 'openrouter') {
                endpoint = 'https://openrouter.ai/api/v1/chat/completions';
            }

            const headers: Record<string, string> = {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${apiKey}`
            };

            if (provider === 'openrouter') {
                headers['HTTP-Referer'] = window.location.origin;
                headers['X-Title'] = 'Helios Forensics';
            }

            const res = await fetch(endpoint, {
                method: 'POST',
                headers,
                body: JSON.stringify({ model, messages })
            });
            if (!res.ok) {
                const errData = await res.json().catch(() => ({}));
                throw new Error(errData.error?.message || `HTTP ${res.status} API Error`);
            }
            const data = await res.json();
            reply = data.choices[0].message.content;

        } else if (provider === 'ollama') {
            const res = await fetch('http://localhost:11434/api/chat', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ model: modelName, messages, stream: false })
            });
            if (!res.ok) throw new Error('API Error');
            const data = await res.json();
            reply = data.message.content;
        }

        chatHistoryStore.update(h => [...h, { role: 'model', text: reply }]);
    } catch (e: any) {
        console.error("AI Chat Error:", e);
        chatHistoryStore.update(h => [...h, { role: 'model', text: `API Error: ${e.message || 'Check console for details.'}` }]);
    } finally {
        isGenerating = false;
    }
  }
</script>

{#if event}
<div class="w-96 border-l border-helios-border bg-helios-bg flex flex-col h-full overflow-hidden shadow-2xl z-20 shrink-0">
  <!-- Header -->
  <div class="flex items-center justify-between p-4 border-b border-helios-border bg-helios-surface">
    <h3 class="font-semibold text-helios-text">Log Inspector</h3>
    <button onclick={onClose} class="text-helios-muted hover:text-helios-text transition-colors" aria-label="Close Inspector">
      <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>

  <div class="flex-1 overflow-y-auto p-4 space-y-6 flex flex-col">
    {#if aiMode === 'idle'}
      <button onclick={openAI} class="w-full shrink-0 py-2.5 bg-purple-500/10 text-purple-400 border border-purple-500/30 rounded-lg flex items-center justify-center gap-2 hover:bg-purple-500/20 transition-colors font-bold text-sm shadow-[0_0_15px_rgba(168,85,247,0.15)]">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
        Analyze with AI
      </button>
    <!-- Meta fields -->
    <div class="space-y-3">
      <div>
        <div class="text-xs font-semibold text-helios-muted mb-1">TIMESTAMP</div>
        <div class="font-mono text-sm text-helios-text">{formatTime(event.timestamp)}</div>
      </div>
      <div>
        <div class="text-xs font-semibold text-helios-muted mb-1">SEVERITY</div>
        <span class="inline-block px-2 py-0.5 text-xs font-semibold border rounded {severityBg(event.severity)}">
          {event.severity ?? '—'}
        </span>
      </div>
      <div>
        <div class="text-xs font-semibold text-helios-muted mb-1">HOSTNAME</div>
        <div class="font-mono text-sm text-helios-text">{event.hostname ?? '—'}</div>
      </div>
      <div>
        <div class="text-xs font-semibold text-helios-muted mb-1">SERVICE</div>
        <div class="font-mono text-sm text-helios-accent">{event.service ?? '—'}</div>
      </div>
    </div>

    <!-- Message -->
    <div>
      <div class="text-xs font-semibold text-helios-muted mb-2">NORMALIZED MESSAGE</div>
      <div class="p-3 text-sm rounded-lg bg-helios-surface-2 border border-helios-border text-helios-text whitespace-pre-wrap break-all">
        {event.message}
      </div>
    </div>

    <!-- Raw Event -->
    <div>
      <div class="text-xs font-semibold text-helios-muted mb-2">RAW EVENT</div>
      <pre class="p-3 overflow-x-auto text-xs rounded-lg bg-[#11111a] border border-helios-border font-mono text-helios-muted whitespace-pre-wrap break-all">{event.raw_event}</pre>
    </div>
    
    <!-- JSON Normalized -->
    <div>
      <div class="text-xs font-semibold text-helios-muted mb-2">JSON NORMALIZED</div>
      <pre class="p-3 overflow-x-auto text-xs rounded-lg bg-[#11111a] border border-helios-border font-mono text-green-400 whitespace-pre-wrap">{JSON.stringify({
        timestamp: event.timestamp,
        hostname: event.hostname,
        service: event.service,
        severity: event.severity,
        message: event.message
      }, null, 2)}</pre>
    </div>
    {:else if aiMode === 'warning'}
      <div class="flex-1 flex flex-col justify-center animate-in fade-in zoom-in duration-200">
        <div class="p-5 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-500 shadow-lg shadow-amber-500/5">
          <div class="flex items-center gap-3 mb-4">
            <svg class="w-6 h-6 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" /></svg>
            <h3 class="font-bold text-lg">Security Warning</h3>
          </div>
          <p class="text-sm text-amber-500/80 mb-6 leading-relaxed">
            Proceeding will transmit this raw log event to an external AI provider (unless using local Ollama).
            <br/><br/>
            Logs frequently contain <strong class="text-amber-400">sensitive PII, internal IP routing, or active credentials.</strong> 
            Please verify no confidential data is exposed before passing this payload to a third party.
          </p>
          <div class="flex flex-col gap-3">
            <button class="w-full py-2 bg-amber-500 hover:bg-amber-400 text-black font-bold rounded-lg transition-colors" onclick={acceptWarning}>
              I Understand, Proceed
            </button>
            <button class="w-full py-2 bg-transparent hover:bg-amber-500/10 text-amber-500 font-semibold rounded-lg transition-colors" onclick={() => aiMode = 'idle'}>
              Cancel
            </button>
          </div>
        </div>
      </div>
    {:else if aiMode === 'chat'}
      <div class="flex-1 flex flex-col h-full rounded-xl overflow-hidden border border-purple-500/30 bg-[#0A0A0F] shadow-[0_0_20px_rgba(168,85,247,0.1)] animate-in fade-in slide-in-from-right-4 duration-300">
        <!-- Settings Header -->
        <div class="bg-purple-500/15 p-3 text-sm font-bold text-purple-400 border-b border-purple-500/30 flex justify-between items-center shrink-0">
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" /></svg>
            AI Forensics
          </span>
          <button onclick={() => aiMode = 'idle'} class="text-purple-400/60 hover:text-purple-400 text-xs uppercase tracking-wider">Close</button>
        </div>
        
        <!-- Settings Bar -->
        <div class="p-3 border-b border-purple-500/20 bg-helios-surface flex flex-col gap-2 shrink-0">
          <div class="flex gap-2">
            <select class="bg-[#11111a] border border-purple-500/30 text-purple-300 text-xs rounded px-2 py-1 outline-none flex-1" value={provider} onchange={(e) => switchProvider(e.currentTarget.value)}>
              <option value="gemini">Google Gemini</option>
              <option value="groq">Groq (Llama 3)</option>
              <option value="openai">OpenAI (GPT-4o/3.5)</option>
              <option value="openrouter">OpenRouter</option>
              <option value="ollama">Ollama (Local)</option>
            </select>
            <button class="px-2 py-1 bg-purple-500/10 text-purple-400 border border-purple-500/30 rounded text-xs hover:bg-purple-500/20" onclick={() => chatHistoryStore.set([])}>
              Clear Chat
            </button>
          </div>
          <div class="flex gap-2">
            <input type="text" placeholder="Model Name" bind:value={modelName} class="w-1/3 bg-[#11111a] border border-purple-500/30 rounded px-2 py-1 text-xs text-purple-300 focus:outline-none focus:border-purple-500/60" title="Model ID" />
            {#if provider !== 'ollama'}
              <input type="password" placeholder="Enter {provider.toUpperCase()} API Key" bind:value={apiKey} onchange={saveSettings}
                class="flex-1 bg-[#11111a] border border-purple-500/30 rounded px-2 py-1 text-xs text-helios-text focus:outline-none focus:border-purple-500/60" />
            {/if}
          </div>
        </div>

        <div class="flex-1 p-4 overflow-y-auto space-y-4 flex flex-col">
          <div class="bg-helios-surface-2 p-3 rounded-lg border border-helios-border text-xs text-helios-muted text-center">
            Context: Selected log event ({formatTime(event.timestamp)}) loaded into AI memory.
          </div>
          
          {#if chatHistory.length === 0}
            <div class="bg-purple-500/10 p-3 rounded-lg border border-purple-500/30 text-purple-300 text-sm">
              <strong class="text-purple-400 block mb-1">AI Assistant:</strong> 
              I'm ready to analyze this log event. Would you like me to extract IOCs, explain the error, or parse custom fields?
            </div>
          {/if}
          
          {#each chatHistory as msg}
            <div class="p-3 rounded-lg border text-sm {msg.role === 'model' ? 'bg-purple-500/10 border-purple-500/30 text-purple-300' : 'bg-helios-surface-2 border-helios-border text-helios-text ml-4'}">
              <strong class="{msg.role === 'model' ? 'text-purple-400' : 'text-helios-muted'} block mb-1 text-xs uppercase tracking-wider">
                {msg.role === 'model' ? 'AI Assistant' : 'You'}
              </strong>
              {#if msg.role === 'model'}
                <div class="prose prose-invert prose-sm max-w-none text-xs text-purple-300 prose-pre:bg-[#0a0a0f] prose-pre:border prose-pre:border-purple-500/30 prose-a:text-purple-400 prose-p:leading-relaxed prose-code:text-purple-200">
                  {@html marked.parse(msg.text)}
                </div>
              {:else}
                <div class="whitespace-pre-wrap text-xs">{msg.text}</div>
              {/if}
            </div>
          {/each}

          {#if isGenerating}
            <div class="p-3 rounded-lg border bg-purple-500/10 border-purple-500/30 text-purple-400 text-sm animate-pulse flex items-center gap-2">
              <span class="w-1.5 h-1.5 rounded-full bg-purple-400 animate-bounce"></span>
              <span class="w-1.5 h-1.5 rounded-full bg-purple-400 animate-bounce delay-100"></span>
              <span class="w-1.5 h-1.5 rounded-full bg-purple-400 animate-bounce delay-200"></span>
            </div>
          {/if}
        </div>
        
        <div class="p-3 border-t border-helios-border bg-helios-surface shrink-0">
          <input type="text" disabled={(provider !== 'ollama' && !apiKey) || isGenerating} bind:value={chatInput} 
            placeholder={(provider !== 'ollama' && !apiKey) ? "Provide API key above first" : "Ask the AI..."}
            onkeydown={(e) => { if (e.key === 'Enter') sendMessage(); }}
            class="w-full bg-[#11111a] border border-helios-border rounded-lg px-4 py-2 text-sm text-helios-text focus:outline-none focus:border-purple-500/50 transition-colors placeholder:text-helios-muted/50 disabled:opacity-50">
        </div>
      </div>
    {/if}
  </div>
</div>
{/if}

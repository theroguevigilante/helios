import { writable } from 'svelte/store';

export type ChatMessage = { role: 'user' | 'model', text: string };

export const chatHistoryStore = writable<ChatMessage[]>([]);

import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

/** Mirrors Rust `Conversation` (serde snake_case). */
export interface Conversation {
  id: string;
  title: string;
  updated_at: number;
}

/** Mirrors Rust `ChatMessage` (serde snake_case). */
export interface ChatMessage {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  tool_calls_json: string | null;
  created_at: number;
}

const LAST_ACTIVE_KEY = 'tbox.chat.lastActiveId';

function newDraftId(): string {
  return `draft-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

/**
 * Chat conversation list + current thread.
 * Empty drafts stay client-only until the first `append_user_message`.
 */
export const useConversationsStore = defineStore('conversations', {
  state: () => ({
    items: [] as Conversation[],
    activeId: null as string | null,
    draftId: null as string | null,
    messages: [] as ChatMessage[],
    isLoadingList: false,
    isLoadingMessages: false,
    isSending: false,
    lastError: null as string | null,
  }),
  getters: {
    isDraft: (state) => state.draftId != null && state.activeId == null,
    hasMessages: (state) => state.messages.length > 0,
  },
  actions: {
    newChat() {
      this.draftId = newDraftId();
      this.activeId = null;
      this.messages = [];
      this.lastError = null;
      try {
        sessionStorage.removeItem(LAST_ACTIVE_KEY);
      } catch {
        /* ignore */
      }
    },

    async loadList() {
      this.isLoadingList = true;
      this.lastError = null;
      try {
        this.items = await invoke<Conversation[]>('list_conversations');
      } catch (error) {
        console.error('[conversations] list failed:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      } finally {
        this.isLoadingList = false;
      }
    },

    async openConversation(id: string) {
      this.isLoadingMessages = true;
      this.lastError = null;
      this.draftId = null;
      this.activeId = id;
      try {
        this.messages = await invoke<ChatMessage[]>('get_conversation_messages', {
          conversationId: id,
        });
        try {
          sessionStorage.setItem(LAST_ACTIVE_KEY, id);
        } catch {
          /* ignore */
        }
      } catch (error) {
        console.error('[conversations] open failed:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
        this.messages = [];
      } finally {
        this.isLoadingMessages = false;
      }
    },

    async deleteConversation(id: string) {
      this.lastError = null;
      try {
        await invoke('delete_conversation', { conversationId: id });
        if (this.activeId === id) {
          this.newChat();
        }
        await this.loadList();
      } catch (error) {
        console.error('[conversations] delete failed:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      }
    },

    /**
     * Persist a user message. Creates the conversation on first send
     * (conversationId null). Draft never appears in `items` beforehand.
     */
    async appendUser(content: string) {
      const trimmed = content.trim();
      if (!trimmed) return;
      this.isSending = true;
      this.lastError = null;
      try {
        const [conversation, message] = await invoke<[Conversation, ChatMessage]>(
          'append_user_message',
          { conversationId: this.activeId, content: trimmed },
        );
        this.draftId = null;
        this.activeId = conversation.id;
        this.messages = [...this.messages, message];
        try {
          sessionStorage.setItem(LAST_ACTIVE_KEY, conversation.id);
        } catch {
          /* ignore */
        }
        await this.loadList();
      } catch (error) {
        console.error('[conversations] append failed:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
        throw error;
      } finally {
        this.isSending = false;
      }
    },

    /** Restore last active chat from sessionStorage (Task 3.3). */
    async restoreLastActive() {
      let last: string | null = null;
      try {
        last = sessionStorage.getItem(LAST_ACTIVE_KEY);
      } catch {
        last = null;
      }
      await this.loadList();
      if (last && this.items.some((c) => c.id === last)) {
        await this.openConversation(last);
        return;
      }
      if (!this.activeId && !this.draftId) {
        this.newChat();
      }
    },
  },
});

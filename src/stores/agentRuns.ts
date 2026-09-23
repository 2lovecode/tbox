import { defineStore } from 'pinia';
import {
  appendReasoning,
  appendText,
  type TrajectoryStep,
} from '@/utils/trajectory';

export type RunStatus = 'idle' | 'running';

export const useAgentRunsStore = defineStore('agentRuns', {
  state: () => ({
    runStatusById: {} as Record<string, RunStatus>,
    liveById: {} as Record<string, TrajectoryStep[]>,
    streamModeById: {} as Record<string, 'live' | 'fallback' | null>,
  }),
  getters: {
    isRunning: (state) => (id: string | null | undefined) =>
      !!id && state.runStatusById[id] === 'running',
    liveSteps: (state) => (id: string | null | undefined): TrajectoryStep[] =>
      (id && state.liveById[id]) || [],
    streamMode: (state) => (id: string | null | undefined) =>
      (id && state.streamModeById[id]) || null,
  },
  actions: {
    setStatus(id: string, status: RunStatus) {
      this.runStatusById = { ...this.runStatusById, [id]: status };
    },

    resetLive(id: string) {
      this.liveById = { ...this.liveById, [id]: [] };
      this.streamModeById = { ...this.streamModeById, [id]: null };
    },

    applyAgentEvent(
      id: string,
      type: string,
      payload: {
        text?: string;
        id?: string;
        args?: unknown;
        result?: string;
        mode?: string;
      },
    ) {
      const steps = [...(this.liveById[id] ?? [])];
      switch (type) {
        case 'stream_meta':
          this.streamModeById = {
            ...this.streamModeById,
            [id]: payload.mode === 'live' ? 'live' : 'fallback',
          };
          break;
        case 'reasoning':
          this.liveById = {
            ...this.liveById,
            [id]: appendReasoning(steps, payload.text ?? ''),
          };
          break;
        case 'token':
          this.liveById = {
            ...this.liveById,
            [id]: appendText(steps, payload.text ?? ''),
          };
          break;
        case 'tool_start':
          this.liveById = {
            ...this.liveById,
            [id]: [
              ...steps,
              {
                type: 'tool',
                id: payload.id ?? 'tool',
                args: payload.args,
                status: 'running',
              },
            ],
          };
          break;
        case 'tool_end': {
          const next = [...steps];
          for (let i = next.length - 1; i >= 0; i--) {
            const s = next[i];
            if (s.type === 'tool' && s.id === payload.id && s.status === 'running') {
              next[i] = { ...s, result: payload.result, status: 'done' };
              break;
            }
          }
          this.liveById = { ...this.liveById, [id]: next };
          break;
        }
        default:
          break;
      }
    },

    clearLive(id: string) {
      const { [id]: _removed, ...rest } = this.liveById;
      this.liveById = rest;
      const { [id]: _m, ...modes } = this.streamModeById;
      this.streamModeById = modes;
    },

    clearConversation(id: string) {
      this.clearLive(id);
      const { [id]: _, ...rest } = this.runStatusById;
      this.runStatusById = rest;
    },
  },
});

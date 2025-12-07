import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { AutomationStatus, type AutomationState, type Mode } from '../common';

export default function useAutomation() {
  const [state, setState] = useState<AutomationState>();
  const [loading, setLoading] = useState(false);
  console.log(state?.history.at(-1));

  useEffect(() => {
    invoke<AutomationState>('get_state').then(setState);
    const unlisten = listen<AutomationState>('stateUpdated', state =>
      setState(state.payload),
    );

    return () => void unlisten.then(unlisten => unlisten());
  }, []);

  return {
    state,
    agentMessage:
      state?.status === 'Running'
        ? (state.history.at(-1)?.action ?? 'Working on your task...')
        : (state?.status ?? 'Idle'),
    loading,
    startAutomation: useCallback(async (instruction: string, mode: Mode) => {
      setLoading(true);
      setState(undefined);
      const sessionId = crypto.randomUUID();
      try {
        await invoke('start_session', {
          sessionId,
          instruction,
          mode,
        });
      } catch (err) {
        setState(state => ({
          ...state,
          session_id: sessionId,
          created_at: new Date().toISOString(),
          instruction,
          status: AutomationStatus.Error,
          history: [],
          error:
            (err as Error)?.stack ??
            (err as Error)?.message ??
            JSON.stringify(err),
        }));
      } finally {
        setLoading(false);
      }
    }, []),
    stopAutomation: useCallback(() => invoke<void>('stop_session'), []),
  };
}

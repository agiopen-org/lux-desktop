import React, { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { AutomationStatus, Mode } from '../common';
import {
  AgentLogo,
  ArrowIcon,
  Instruction,
  InstructionEditor,
  ModeSelector,
  Options,
  RunningIcon,
  Timeline,
} from './components';
import { store } from './utils';
import useAutomation from './useAutomation';

const App: React.FC = () => {
  const { state, agentMessage, loading, startAutomation, stopAutomation } =
    useAutomation();
  const { status, history, error } = state ?? {};
  const isRunning = status === AutomationStatus.Running;
  const historyAvailable = !!history?.length && !isRunning;

  const [mode, setMode] = useState(Mode.Actor);
  const [showHistory, setShowHistory] = useState(false);
  const [currentInstruction, setCurrentInstruction] = useState('');
  const instruction = state?.instruction || currentInstruction;

  useEffect(
    () =>
      void store.get<Mode>('mode').then(mode => setMode(mode ?? Mode.Actor)),
    [],
  );
  useEffect(() => {
    getCurrentWindow().setContentProtected(isRunning);
    setShowHistory(false);
  }, [isRunning]);

  return (
    <div className="flex size-full flex-col bg-accent-b p-2">
      {/* Chat Window */}
      <div className="flex flex-col size-full overflow-hidden rounded-chat bg-white">
        {/* Top Bar */}
        <div className="flex justify-between border-b-1 border-accent-b p-3 pb-2">
          <ModeSelector
            mode={mode}
            onChange={async mode => {
              setMode(mode);
              await store.set('mode', mode);
            }}
          />
          <Options />
        </div>

        {/* Content Area */}
        <div className="flex flex-auto flex-col justify-between  min-h-0 p-3">
          {instruction ? (
            // Chat Bubbles - show when running
            <div className="flex min-h-36 flex-col overflow-y-scroll p-3">
              {/* User message */}
              <div className="flex flex-col items-end gap-2.5 pb-5">
                <div className="max-w-md rounded-bl-2xl rounded-tl-2xl rounded-tr-2xl bg-primary-light-3 p-2">
                  <p className="leading-chat text-primary-dark-2">
                    <Instruction mode={mode} instruction={instruction} />
                  </p>
                </div>
              </div>

              {/* Agent progress */}
              <div className="flex flex-col items-start gap-2.5">
                <div className="flex items-center gap-2.5">
                  {/* Lightning icon */}
                  <div className="h-7.5 w-7.5 shrink-0">
                    <AgentLogo
                      className="text-primary"
                      completed={status === AutomationStatus.Completed}
                    />
                  </div>
                  <div className="flex items-center rounded-chat bg-accent-b px-3 py-2">
                    {status === AutomationStatus.Idle ? (
                      <RunningIcon />
                    ) : (
                      <span className="px-2 text-base text-accent-b-0 text-gray-700">
                        {agentMessage}
                      </span>
                    )}
                    {historyAvailable && (
                      <button
                        onClick={() => setShowHistory(s => !s)}
                        className="flex h-7 w-7 items-center justify-center rounded-full text-accent-c-2 hover:bg-gray-300"
                      >
                        <ArrowIcon direction={showHistory ? 'up' : 'down'} />
                      </button>
                    )}
                  </div>
                </div>
              </div>
              {isRunning && <RunningIcon />}
              {historyAvailable && (
                <Timeline open={showHistory} history={history} />
              )}
            </div>
          ) : (
            <div />
          )}

          <div className="flex-initial">
            {error && (
              <div className="px-2 my-2 bg-error/6 text-error flex items-center rounded-lg">
                <div className="">
                  <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                  >
                    <path
                      d="M12 15.75V16.275M12 9V13.9125M4.875 12.75C4.875 16.685 8.06497 19.875 12 19.875C15.935 19.875 19.125 16.685 19.125 12.75C19.125 8.81497 15.935 5.625 12 5.625C8.06497 5.625 4.875 8.81497 4.875 12.75Z"
                      stroke="#E8484B"
                      strokeLinecap="round"
                    />
                  </svg>
                </div>
                <span className="py-2">{error}</span>
              </div>
            )}
            {/* Prompt Box */}
            <InstructionEditor
              mode={mode}
              loading={loading}
              status={status}
              startAutomation={async instruction => {
                if (mode === Mode.Tasker) {
                  const taskMode = `tasker:${instruction.slice(1).trim()}`;
                  setCurrentInstruction(instruction);
                  await startAutomation('', taskMode as Mode);
                  return;
                }
                setCurrentInstruction(instruction);
                await startAutomation(instruction, mode);
              }}
              stopAutomation={stopAutomation}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;

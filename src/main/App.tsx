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
  const [banner, setBanner] = useState(true);

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
    <div className="flex flex-col size-full bg-accent-b">
      {banner && (
        <a
          className="w-full flex items-center justify-center text-white bg-[#4EACDB] gap-2.5 py-1"
          href="https://discord.gg/PVAtX8PzxK"
          target="_blank"
        >
          <div className="flex items-center gap-1">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M19.2701 5.33C17.9401 4.71 16.5001 4.26 15.0001 4C14.9737 4.00038 14.9486 4.01116 14.9301 4.03C14.7501 4.36 14.5401 4.79 14.4001 5.12C12.8091 4.88015 11.1911 4.88015 9.60012 5.12C9.46012 4.78 9.25012 4.36 9.06012 4.03C9.05012 4.01 9.02012 4 8.99012 4C7.49012 4.26 6.06012 4.71 4.72012 5.33C4.71012 5.33 4.70012 5.34 4.69012 5.35C1.97012 9.42 1.22012 13.38 1.59012 17.3C1.59012 17.32 1.60012 17.34 1.62012 17.35C3.42012 18.67 5.15012 19.47 6.86012 20C6.89012 20.01 6.92012 20 6.93012 19.98C7.33012 19.43 7.69012 18.85 8.00012 18.24C8.02012 18.2 8.00012 18.16 7.96012 18.15C7.39012 17.93 6.85012 17.67 6.32012 17.37C6.28012 17.35 6.28012 17.29 6.31012 17.26C6.42012 17.18 6.53012 17.09 6.64012 17.01C6.66012 16.99 6.69012 16.99 6.71012 17C10.1501 18.57 13.8601 18.57 17.2601 17C17.2801 16.99 17.3101 16.99 17.3301 17.01C17.4401 17.1 17.5501 17.18 17.6601 17.27C17.7001 17.3 17.7001 17.36 17.6501 17.38C17.1301 17.69 16.5801 17.94 16.0101 18.16C15.9701 18.17 15.9601 18.22 15.9701 18.25C16.2901 18.86 16.6501 19.44 17.0401 19.99C17.0701 20 17.1001 20.01 17.1301 20C18.8501 19.47 20.5801 18.67 22.3801 17.35C22.4001 17.34 22.4101 17.32 22.4101 17.3C22.8501 12.77 21.6801 8.84 19.3101 5.35C19.3001 5.34 19.2901 5.33 19.2701 5.33ZM8.52012 14.91C7.49012 14.91 6.63012 13.96 6.63012 12.79C6.63012 11.62 7.47012 10.67 8.52012 10.67C9.58012 10.67 10.4201 11.63 10.4101 12.79C10.4101 13.96 9.57012 14.91 8.52012 14.91ZM15.4901 14.91C14.4601 14.91 13.6001 13.96 13.6001 12.79C13.6001 11.62 14.4401 10.67 15.4901 10.67C16.5501 10.67 17.3901 11.63 17.3801 12.79C17.3801 13.96 16.5501 14.91 15.4901 14.91Z" fill="white"/>
            </svg>
            <span className="font-medium">
              We’re on Discord
            </span>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M6.83415 3.83124L5 3.83123C4.17158 3.83123 3.5 4.5028 3.5 5.33123L3.5 11C3.5 11.8284 4.17157 12.5 5 12.5L10.6688 12.5C11.4972 12.5 12.1688 11.8284 12.1688 11V9.16587M7.16756 8.83245L12.1688 3.83123M9.50143 3.83121L12.1688 3.83609L12.1688 6.4985" stroke="white" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </div>
          <span className="hidden md:block">
            Click here and join to share builds, report bugs, and help shape Lux with us.
          </span>
          <span className="absolute right-[20px]" onClick={e => { e.stopPropagation(); setBanner(false) }}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M5.25 5.25L12 12M12 12L5.25 18.75M12 12L18.75 18.75M12 12L18.75 5.25" stroke="white" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </span>
        </a>
      )}
      <div className="flex flex-1 flex-col p-2">
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
    </div>
  );
};

export default App;

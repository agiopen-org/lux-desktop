import React, { useState } from 'react';
import { Mode } from '../../common';
import Dropdown from './Dropdown';

export interface ModeSelectorProps {
  mode: Mode;
  onChange: (mode: Mode) => void;
}

const modes = [
  {
    id: Mode.Actor,
    label: 'Actor',
    description: 'Actor mode lets you perform short tasks at lightning speed',
  },
  {
    id: Mode.Thinker,
    label: 'Thinker',
    description:
      'Thinker mode lets you perform complex, long-horizon tasks that can take anywhere from several minutes to hours',
  },
  {
    id: Mode.Tasker,
    label: 'Tasker',
    description:
      'Tasker mode lets you define the exact steps the agent takes, offering exceptional control',
  },
];

const ModeSelector: React.FC<ModeSelectorProps> = ({ mode, onChange }) => {
  const [open, setOpen] = useState(false);
  const selectedMode = modes.find(m => m.id === mode)!;

  return (
    <div>
      <Dropdown
        open={open}
        setOpen={setOpen}
        options={modes.map(m => ({
          key: m.id,
          label: m.label,
          selected: m.id === mode,
          onClick: () => onChange(m.id),
        }))}
        position="bottom-right"
      >
        <button
          onClick={() => setOpen(!open)}
          className="hover:text-accent-b-neg1 hover:bg-accent-b rounded-full px-2.5 py-0.5 -ml-2.5 flex items-center gap-1 text-sm-chat text-accent-b-0 text-gray-700"
        >
          <span>{selectedMode.label}</span>
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            className={`transition-transform ${open ? 'rotate-180' : ''}`}
          >
            <path
              d="M4 6L8 10L12 6"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
      </Dropdown>

      <span className="text-accent-b-neg-1 text-sm">
        {selectedMode.description}
      </span>
    </div>
  );
};

export default ModeSelector;

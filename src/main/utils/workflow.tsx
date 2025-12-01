export interface Workflow {
  key: string;
  command: string;
  icon: React.ReactNode;
}

export const workflows: Workflow[] = [
  {
    key: 'software_qa',
    command: 'Software QA',
    icon: null,
  },
  {
    key: 'cvs_appointment',
    command: 'CVS Appointment',
    icon: null,
  },
];

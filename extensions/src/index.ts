export interface ExtensionInput {
  command: string;
  args: Record<string, unknown>;
  config: FlowConfig;
  cwd: string;
}

export interface ExtensionOutput {
  success: boolean;
  data?: unknown;
  error?: string;
  messages?: Message[];
}

export interface FlowConfig {
  projectsDir: string;
  stateDir: string;
  defaultBaseBranch: string;
}

export interface Message {
  level: 'info' | 'warn' | 'error';
  text: string;
}

export { SemgrepScanner } from './scanners/semgrep.js';
export { TrivyScanner } from './scanners/trivy.js';

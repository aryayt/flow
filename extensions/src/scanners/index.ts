export { SemgrepScanner } from './semgrep.js';
export { TrivyScanner } from './trivy.js';

export interface ScanResult {
  scanner: string;
  findings: Finding[];
  summary: {
    critical: number;
    high: number;
    medium: number;
    low: number;
  };
}

export interface Finding {
  severity: 'critical' | 'high' | 'medium' | 'low' | 'info';
  title: string;
  description: string;
  file?: string;
  line?: number;
  rule?: string;
}

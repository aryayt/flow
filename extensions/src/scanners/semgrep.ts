import { spawn } from 'node:child_process';
import type { Finding, ScanResult } from './index.js';

export class SemgrepScanner {
  private config: string;

  constructor(config = 'auto') {
    this.config = config;
  }

  async scan(targetDir: string): Promise<ScanResult> {
    const findings: Finding[] = [];

    try {
      const output = await this.runSemgrep(targetDir);
      const results = JSON.parse(output);

      for (const result of results.results || []) {
        findings.push({
          severity: this.mapSeverity(result.extra?.severity || 'info'),
          title: result.check_id || 'Unknown',
          description: result.extra?.message || '',
          file: result.path,
          line: result.start?.line,
          rule: result.check_id,
        });
      }
    } catch (error) {
      console.error('Semgrep scan failed:', error);
    }

    return {
      scanner: 'semgrep',
      findings,
      summary: this.summarize(findings),
    };
  }

  private runSemgrep(targetDir: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const process = spawn('semgrep', ['scan', '--config', this.config, '--json', targetDir]);

      let stdout = '';
      let stderr = '';

      process.stdout.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      process.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      process.on('close', (code) => {
        if (code === 0 || code === 1) {
          // Semgrep returns 1 if findings exist
          resolve(stdout);
        } else {
          reject(new Error(`Semgrep exited with code ${code}: ${stderr}`));
        }
      });

      process.on('error', reject);
    });
  }

  private mapSeverity(severity: string): Finding['severity'] {
    const map: Record<string, Finding['severity']> = {
      ERROR: 'critical',
      WARNING: 'high',
      INFO: 'medium',
    };
    return map[severity.toUpperCase()] || 'low';
  }

  private summarize(findings: Finding[]): ScanResult['summary'] {
    return {
      critical: findings.filter((f) => f.severity === 'critical').length,
      high: findings.filter((f) => f.severity === 'high').length,
      medium: findings.filter((f) => f.severity === 'medium').length,
      low: findings.filter((f) => f.severity === 'low').length,
    };
  }
}

import { spawn } from 'node:child_process';
import type { Finding, ScanResult } from './index.js';

export class TrivyScanner {
  private severities: string[];

  constructor(severities = ['CRITICAL', 'HIGH', 'MEDIUM', 'LOW']) {
    this.severities = severities;
  }

  async scan(targetDir: string): Promise<ScanResult> {
    const findings: Finding[] = [];

    try {
      const output = await this.runTrivy(targetDir);
      const results = JSON.parse(output);

      for (const result of results.Results || []) {
        for (const vuln of result.Vulnerabilities || []) {
          findings.push({
            severity: this.mapSeverity(vuln.Severity || 'UNKNOWN'),
            title: vuln.VulnerabilityID || 'Unknown',
            description: vuln.Title || vuln.Description || '',
            file: result.Target,
            rule: vuln.VulnerabilityID,
          });
        }
      }
    } catch (error) {
      console.error('Trivy scan failed:', error);
    }

    return {
      scanner: 'trivy',
      findings,
      summary: this.summarize(findings),
    };
  }

  private runTrivy(targetDir: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const process = spawn('trivy', [
        'fs',
        '--format',
        'json',
        '--severity',
        this.severities.join(','),
        targetDir,
      ]);

      let stdout = '';
      let stderr = '';

      process.stdout.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      process.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      process.on('close', (code) => {
        if (code === 0) {
          resolve(stdout);
        } else {
          reject(new Error(`Trivy exited with code ${code}: ${stderr}`));
        }
      });

      process.on('error', reject);
    });
  }

  private mapSeverity(severity: string): Finding['severity'] {
    const map: Record<string, Finding['severity']> = {
      CRITICAL: 'critical',
      HIGH: 'high',
      MEDIUM: 'medium',
      LOW: 'low',
    };
    return map[severity.toUpperCase()] || 'info';
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

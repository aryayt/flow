import type { ExtensionInput, ExtensionOutput } from '../index.js';

export async function postSwitch(input: ExtensionInput): Promise<ExtensionOutput> {
  const { args, cwd, config } = input;
  const projectName = args.project as string | undefined;

  console.info(`Post-switch hook triggered`);
  console.info(`Working directory: ${cwd}`);
  console.info(`Config:`, config);

  const messages: ExtensionOutput['messages'] = [];

  if (projectName) {
    messages.push({
      level: 'info',
      text: `Switched to project: ${projectName}`,
    });
  }

  // Check for project-specific configuration
  const projectConfig = await checkProjectConfig(cwd);
  if (projectConfig) {
    messages.push({
      level: 'info',
      text: `Found project configuration`,
    });
  }

  return {
    success: true,
    messages,
    data: {
      projectName,
      hasConfig: projectConfig !== null,
    },
  };
}

async function checkProjectConfig(cwd: string): Promise<Record<string, unknown> | null> {
  const fs = await import('node:fs/promises');
  const path = await import('node:path');

  const configPath = path.join(cwd, '.flow.json');

  try {
    const content = await fs.readFile(configPath, 'utf-8');
    return JSON.parse(content) as Record<string, unknown>;
  } catch {
    return null;
  }
}

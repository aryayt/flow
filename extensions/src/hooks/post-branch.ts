import type { ExtensionInput, ExtensionOutput } from '../index.js';

export async function postBranch(input: ExtensionInput): Promise<ExtensionOutput> {
  const { args, cwd } = input;
  const branchName = args.name as string;

  console.info(`Post-branch hook triggered for: ${branchName}`);
  console.info(`Working directory: ${cwd}`);

  // Example: Set up branch-specific environment
  const messages: ExtensionOutput['messages'] = [];

  // Check if this is a feature branch
  if (branchName.startsWith('feat/') || branchName.startsWith('feature/')) {
    messages.push({
      level: 'info',
      text: `Feature branch detected: ${branchName}`,
    });
  }

  // Check if this is a fix branch
  if (branchName.startsWith('fix/') || branchName.startsWith('bugfix/')) {
    messages.push({
      level: 'info',
      text: `Fix branch detected: ${branchName}`,
    });
  }

  return {
    success: true,
    messages,
    data: {
      branchName,
      branchType: detectBranchType(branchName),
    },
  };
}

function detectBranchType(name: string): string {
  if (name.startsWith('feat/') || name.startsWith('feature/')) return 'feature';
  if (name.startsWith('fix/') || name.startsWith('bugfix/')) return 'fix';
  if (name.startsWith('chore/')) return 'chore';
  if (name.startsWith('docs/')) return 'docs';
  if (name.startsWith('refactor/')) return 'refactor';
  if (name.startsWith('test/')) return 'test';
  return 'other';
}

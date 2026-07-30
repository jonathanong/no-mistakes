import { readFileSync } from 'node:fs';

export const releasePolicy = readFileSync('docs/release-policy.md', 'utf8');
export const releaseWorkflow = readFileSync('.github/workflows/release.yml', 'utf8');

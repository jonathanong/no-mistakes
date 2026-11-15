import { readdirSync } from 'node:fs';

export function scanOverridden(options: { root: string }) {
  return readdirSync(options.root);
}

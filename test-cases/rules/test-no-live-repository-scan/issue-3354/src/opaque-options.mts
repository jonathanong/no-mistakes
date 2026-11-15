import { readdirSync } from 'node:fs';

export function scanOpaque(options: { root: string }) {
  return readdirSync(options.root);
}

import { readdirSync } from 'node:fs';

export function scanMigrations(options: { root: string }) {
  return readdirSync(options.root);
}

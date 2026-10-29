import { readFileSync } from 'node:fs';

export function readWrapped(path: string) {
  return readFileSync(path, 'utf8');
}

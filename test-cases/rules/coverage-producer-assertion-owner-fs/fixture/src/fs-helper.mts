import { readFileSync } from 'node:fs';

export function readResult() {
  return readFileSync('result', 'utf8');
}

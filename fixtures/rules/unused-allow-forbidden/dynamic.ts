import { readFileSync } from 'node:fs';
export function load(path: string) { return readFileSync(path, 'utf8'); }

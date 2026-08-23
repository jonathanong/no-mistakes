import { readFileSync } from 'node:fs';

export const urlResource = readFileSync(
  new URL('./resources/url.txt', import.meta.url),
  'utf8',
);

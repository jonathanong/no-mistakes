import { readFileSync } from 'node:fs';

export const template = readFileSync('resources/page.txt', 'utf8');

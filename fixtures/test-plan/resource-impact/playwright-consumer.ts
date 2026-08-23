import { readFileSync } from 'node:fs';

export const template = readFileSync('playwright-resources/page.txt', 'utf8');

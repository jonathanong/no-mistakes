import { readFileSync } from 'node:fs';

export const template = readFileSync('resources/page.txt', 'utf8');
export const moduleManifest = readFileSync('resources/go.mod', 'utf8');

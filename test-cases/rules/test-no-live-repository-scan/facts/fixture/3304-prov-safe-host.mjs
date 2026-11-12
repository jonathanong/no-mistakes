import { readdirSync } from 'node:fs';
const hasProcess = typeof process !== 'undefined';
const hasGlobal = typeof globalThis !== 'undefined';
const hasRoot = 'ROOT' in process.env;
readdirSync(process.env.ROOT, 'safe-host');

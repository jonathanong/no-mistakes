import { mkdtempSync, readdirSync } from 'node:fs';
import * as fs from 'fs';
Object.assign(fs, { mkdtempSync: () => process.cwd() });
readdirSync(mkdtempSync('case-'));

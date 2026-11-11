import { mkdtempSync, readdirSync } from 'node:fs';
import * as fs from 'fs';
const key = 'mkdtempSync';
void fs[key];
readdirSync(mkdtempSync('case-'));

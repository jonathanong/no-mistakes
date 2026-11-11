import { mkdtempSync, readdirSync } from 'node:fs';
import * as fs from 'fs';
const local = { spyOn(..._args: unknown[]) {} };
local.spyOn(fs, 'readdirSync');
readdirSync(mkdtempSync('case-'));

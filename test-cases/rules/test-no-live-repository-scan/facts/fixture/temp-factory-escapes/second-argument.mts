import { mkdtempSync, readdirSync } from 'node:fs';
import * as fs from 'fs';
function opaqueCall(..._args: unknown[]) {}
opaqueCall(0, fs);
readdirSync(mkdtempSync('case-'));

import { mkdtempSync, readdirSync } from 'node:fs';
import * as fs from 'fs';
function opaqueCall(..._args: unknown[]) {}
opaqueCall(fs);
readdirSync(mkdtempSync('case-'));

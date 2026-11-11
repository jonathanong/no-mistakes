import { readdirSync, mkdtempSync } from 'node:fs';
import * as escaped from 'node:fs';
import * as overwritten from 'fs';

const alias = escaped;
void alias;
Reflect.set(overwritten, 'mkdtempSync', () => process.cwd());

readdirSync(escaped.mkdtempSync('case-'));
readdirSync(overwritten.mkdtempSync('case-'));
readdirSync(mkdtempSync('case-'));

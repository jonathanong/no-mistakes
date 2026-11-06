import { strictEqual } from 'node:assert';
import { inspect } from './broken';
export function produce() { strictEqual(1, 1); inspect(); }

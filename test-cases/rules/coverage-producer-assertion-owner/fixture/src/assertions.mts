import { strictEqual } from 'node:assert';
export function assertResult() { strictEqual(1, 1); }
export function* generatorAssertion() { strictEqual(1, 1); }

import { strictEqual } from 'node:assert';
import { runThen, runCatch, runFinally } from '../src/script.mts';

export async function assertsThen() { await runThen(); strictEqual(1, 1); }
export async function assertsCatch() { await runCatch(); strictEqual(1, 1); }
export async function assertsFinally() { await runFinally(); strictEqual(1, 1); }

import { runUnresolvedThen, runUnresolvedCatch, runUnresolvedFinally } from '../src/script.mts';
export async function assertsUnresolvedThen() { await runUnresolvedThen(); strictEqual(1, 1); }
export async function assertsUnresolvedCatch() { await runUnresolvedCatch(); strictEqual(1, 1); }
export async function assertsUnresolvedFinally() { await runUnresolvedFinally(); strictEqual(1, 1); }

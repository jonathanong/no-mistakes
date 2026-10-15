import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
// Agreement can discard an intermediate return; a later callee must retain its own evidence.
const a = undefined ?? fallbackA;
const b = undefined ?? fallbackB;
function readB() { return b; }
function readA() { return a; }
function choose(flag: boolean) { if (flag) return a; return independent; }
export function run(input: Executor) {
  const discarded = choose(true);
  const query = discarded ?? b ?? input ?? independent;
  for (const item of ['a', 'b']) { query(item); }
}

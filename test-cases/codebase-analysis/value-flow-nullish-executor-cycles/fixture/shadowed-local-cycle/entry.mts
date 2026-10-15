import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
// Keep lexical shadow and TDZ admission independent of nullable executor evidence.
const a = readB() ?? fallbackA;
const b = readA() ?? fallbackB;
function readB() { return b; }
function readA() { return a; }
export function run(input: Executor) {
 const a = independent;
 const query = input ?? a;
 for (const item of ['a', 'b']) { query(item); }
}

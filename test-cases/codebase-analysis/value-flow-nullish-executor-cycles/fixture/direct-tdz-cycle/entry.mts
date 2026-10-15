import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
// Keep lexical shadow and TDZ admission independent of nullable executor evidence.
const a = b ?? fallbackA;
const b = a ?? fallbackB;
export function run(input: Executor) {
 const query = input ?? a ?? b ?? independent;
 for (const item of ['a', 'b']) { query(item); }
}

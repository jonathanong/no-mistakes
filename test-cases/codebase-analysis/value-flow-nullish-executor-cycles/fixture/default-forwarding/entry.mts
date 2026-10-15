import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
// Keep normalized parameter frames distinct after invocation reads change.
function first(value = second()) { return value ?? fallbackA; }
function second(value = first()) { return value ?? fallbackB; }
export function run(input: Executor) {
 const query = input ?? first() ?? fallbackB ?? independent;
 for (const item of ['a', 'b']) { query(item); }
}

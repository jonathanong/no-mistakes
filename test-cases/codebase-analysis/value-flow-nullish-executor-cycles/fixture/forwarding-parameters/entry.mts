import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
// Keep normalized parameter frames distinct after invocation reads change.
function first(value, other) { return second(value ?? other, value); }
function second(value, other) { return first(value ?? other, value) ?? fallbackA; }
export function run(input: Executor) {
 const query = first(input, fallbackA) ?? fallbackB ?? independent;
 for (const item of ['a', 'b']) { query(item); }
}

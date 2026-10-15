import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
// Keep normalized parameter frames distinct after invocation reads change.
function identity(value) { const selected = value ?? independent; return selected; }
function first(value) { return identity(value ?? fallbackA); }
function second(value) { return identity(value ?? fallbackB); }
export function run(input: Executor) {
 const before = first(input);
 const after = second(input);
 const query = before ?? after ?? independent;
 for (const item of ['a', 'b']) { query(item); }
}

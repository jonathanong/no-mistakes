import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
// The same authored a-read sees branch active on only one acquisition path.
const a = readBranch() ?? fallbackA;
const low = a ?? fallbackB;
const branch = low ?? independent;
function readBranch() { return branch; }
export function run(input: Executor) {
 const first = branch ?? input;
 const second = low ?? input;
 const query = first ?? second ?? independent;
 for (const item of ['a', 'b']) { query(item); }
}

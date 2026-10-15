import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
// The cycle includes the layered aliases, rather than sitting below an acyclic prefix.
const a = readB() ?? fallbackA;
const b = readA() ?? fallbackB;
const x0 = a ?? b;
const y0 = b ?? a;
const x1 = x0 ?? y0;
const y1 = y0 ?? x0;
const x2 = x1 ?? y1;
const y2 = y1 ?? x1;
function readB() { return x2; }
function readA() { return a; }
export function run(input: Executor) {
  const query = input ?? x2 ?? independent;
  for (const item of ['a', 'b']) { query(item); }
}

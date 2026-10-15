import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
// Return bodies follow both bindings so this protects callable cycles, not a TDZ shortcut.
const a = readB() ?? fallbackA;
const b = readA() ?? fallbackB;
function readB() { return b; }
function readA() { return a; }
export function run(input: Executor) {
  const x0 = input ?? a;
  const y0 = b ?? independent;
  const x1 = x0 ?? y0;
  const y1 = y0 ?? x0;
  const x2 = x1 ?? y1;
  const y2 = y1 ?? x1;
  const query = x2 ?? y2 ?? independent;
  for (const item of ['a', 'b']) { query(item); }
}

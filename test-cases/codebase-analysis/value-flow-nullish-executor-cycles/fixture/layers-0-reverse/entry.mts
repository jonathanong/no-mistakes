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
  const x0 = input ?? b;
  const y0 = a ?? independent;
  const query = x0 ?? y0 ?? independent;
  for (const item of ['a', 'b']) { query(item); }
}

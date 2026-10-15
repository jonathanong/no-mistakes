import type { Executor } from './types.mts';
export function fallbackA() {}
export function fallbackB() {}
export function independent() {}
function a() { return b() ?? fallbackA; }
function b() { return a() ?? fallbackB; }
export function run(input: Executor) {
 const x0 = a() ?? input;
 const y0 = b() ?? independent;
 const x1 = x0 ?? y0;
 const y1 = y0 ?? x0;
 const x2 = x1 ?? y1;
 const y2 = y1 ?? x1;
 const query = x2 ?? y2 ?? independent;
 for (const item of ['a', 'b']) { query(item); }
}

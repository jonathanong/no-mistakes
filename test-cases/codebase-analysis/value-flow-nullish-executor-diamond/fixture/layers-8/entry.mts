import type { Executor } from './types.mts';
function sink() {}
// Distinct aliases share earlier nullable branches; keep the layered diamond.
export function run(a: Executor, b: Executor) {
  const x0 = a;
  const y0 = b;
  const x1 = x0 ?? y0;
  const y1 = y0 ?? x0;
  const x2 = x1 ?? y1;
  const y2 = y1 ?? x1;
  const x3 = x2 ?? y2;
  const y3 = y2 ?? x2;
  const x4 = x3 ?? y3;
  const y4 = y3 ?? x3;
  const x5 = x4 ?? y4;
  const y5 = y4 ?? x4;
  const x6 = x5 ?? y5;
  const y6 = y5 ?? x5;
  const x7 = x6 ?? y6;
  const y7 = y6 ?? x6;
  const x8 = x7 ?? y7;
  const y8 = y7 ?? x7;
  const query = x8 ?? sink;
  query();
}

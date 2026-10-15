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
  const x9 = x8 ?? y8;
  const y9 = y8 ?? x8;
  const x10 = x9 ?? y9;
  const y10 = y9 ?? x9;
  const x11 = x10 ?? y10;
  const y11 = y10 ?? x10;
  const x12 = x11 ?? y11;
  const y12 = y11 ?? x11;
  const x13 = x12 ?? y12;
  const y13 = y12 ?? x12;
  const x14 = x13 ?? y13;
  const y14 = y13 ?? x13;
  const x15 = x14 ?? y14;
  const y15 = y14 ?? x14;
  const x16 = x15 ?? y15;
  const y16 = y15 ?? x15;
  const query = x16 ?? sink;
  query();
}

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
 const x17 = x16 ?? y16;
 const y17 = y16 ?? x16;
 const x18 = x17 ?? y17;
 const y18 = y17 ?? x17;
 const x19 = x18 ?? y18;
 const y19 = y18 ?? x18;
 const x20 = x19 ?? y19;
 const y20 = y19 ?? x19;
 const x21 = x20 ?? y20;
 const y21 = y20 ?? x20;
 const x22 = x21 ?? y21;
 const y22 = y21 ?? x21;
 const x23 = x22 ?? y22;
 const y23 = y22 ?? x22;
 const x24 = x23 ?? y23;
 const y24 = y23 ?? x23;
 const x25 = x24 ?? y24;
 const y25 = y24 ?? x24;
 const x26 = x25 ?? y25;
 const y26 = y25 ?? x25;
 const x27 = x26 ?? y26;
 const y27 = y26 ?? x26;
 const x28 = x27 ?? y27;
 const y28 = y27 ?? x27;
 const x29 = x28 ?? y28;
 const y29 = y28 ?? x28;
 const x30 = x29 ?? y29;
 const y30 = y29 ?? x29;
 const x31 = x30 ?? y30;
 const y31 = y30 ?? x30;
 const x32 = x31 ?? y31;
 const y32 = y31 ?? x31;
 const query = x32 ?? y32 ?? independent;
 for (const item of ['a', 'b']) { query(item); }
}

import { read } from "@example/db";

export function atCap(a?: string, b?: string, c?: string, d?: string) {
  return read(
    `SELECT id FROM orders WHERE account_id = $1 ${a ? "/*a1*/" : "/*a0*/"} ${b ? "/*b1*/" : "/*b0*/"} ${c ? "/*c1*/" : "/*c0*/"} ${d ? "/*d1*/" : "/*d0*/"}`,
    [1],
  );
}

export function overCap(a?: string, b?: string, c?: string, d?: string, e?: string) {
  return read(
    `SELECT id FROM orders WHERE account_id = $1 ${a ? "/*a1*/" : "/*a0*/"} ${b ? "/*b1*/" : "/*b0*/"} ${c ? "/*c1*/" : "/*c0*/"} ${d ? "/*d1*/" : "/*d0*/"} ${e ? "/*e1*/" : "/*e0*/"}`,
    [1],
  );
}

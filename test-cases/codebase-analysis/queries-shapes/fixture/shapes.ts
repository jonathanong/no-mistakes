import { f } from "./target";

declare const ident: unknown;
declare const cond: boolean;
declare function other(): unknown;

// Each call exercises a different coarse argument-shape tag. Keep one tag per
// argument position so the expected `args` arrays below stay readable.
export function shapes() {
  f("literal", `template`); // string, string
  f(1, 9007199254740993n); // number, number
  f(true, null); // boolean, null
  f(ident, { key: 1 }); // identifier, object
  f([1, 2], () => 1); // array, arrow
  f(other(), function inner() {}); // call, arrow
  f(cond ? 1 : 2); // other
  // Member-expression callee — not a plain identifier, so it never matches.
  Math.max(1, 2);
}

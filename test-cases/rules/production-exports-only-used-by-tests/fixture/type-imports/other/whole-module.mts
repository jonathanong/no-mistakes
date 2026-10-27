// A bare module type carries no member name, so it stays unresolved.
export type Whole = typeof import("../src/whole-module.mjs");

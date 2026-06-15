// NodeNext/ESM: imports sources via the emitted `.js`/`.mjs`/`.cjs` specifiers.
import { x } from "./dep.js";
import { m } from "./depm.mjs";
import { c } from "./depc.cjs";

export const e = x + m + c;

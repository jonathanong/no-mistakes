// A namespace used as a whole may expose every member, so it stays unresolved.
import type * as Whole from "../src/whole-namespace.mjs";
export type Shape = typeof Whole;

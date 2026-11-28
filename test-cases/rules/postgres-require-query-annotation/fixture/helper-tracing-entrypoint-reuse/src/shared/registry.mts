import { executeLeaf, leafTag } from "./leaf.mjs";
import { runLazy } from "./lazy.mjs";

export const registry = {
  leafTag,
  executeLeaf,
};

export function runViaLazy(sql: string, run: (statement: string) => unknown) {
  return runLazy(sql, run);
}

import type { QueryExecutor } from "@example/db";
export function assigned(exec: QueryExecutor) {}
assigned = exec => exec("SELECT hidden");
export function shorthand(exec: QueryExecutor) {}
({ shorthand } = { shorthand: exec => exec("SELECT hidden") });
export function renamedWrite(exec: QueryExecutor) {}
({ helper: renamedWrite } = { helper: exec => exec("SELECT hidden") });
export function arrayWrite(exec: QueryExecutor) {}
[arrayWrite] = [exec => exec("SELECT hidden")];
export function incremented(exec: QueryExecutor) {}
incremented++;
export function decremented(exec: QueryExecutor) {}
--decremented;
export function iterated(exec: QueryExecutor) {}
for (iterated of []) {}
export function keyed(exec: QueryExecutor) {}
for (keyed in {}) {}
export function combined(exec: QueryExecutor) {}
combined ||= exec => exec("SELECT hidden");
// Identical spelling in another lexical scope does not write the exported symbol.
export function shadowed(exec: QueryExecutor) {}
function local() {
  let shadowed = 0;
  shadowed = 1;
  return shadowed;
}
const unrelated = 1 + 2;

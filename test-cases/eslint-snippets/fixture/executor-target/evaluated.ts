import type { QueryExecutor } from "@example/db";
export function evaluated(exec: QueryExecutor) {}
// Direct eval can replace a function binding without an AST assignment.
eval("evaluated = exec => exec('SELECT hidden')");

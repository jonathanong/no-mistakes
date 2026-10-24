import type { QueryExecutor } from "@example/db";
export function evaluatedWrapped(exec: QueryExecutor) {}
// Parentheses and assertions do not make a direct eval call safe.
((eval as typeof eval))("evaluatedWrapped = exec => exec('SELECT hidden')");

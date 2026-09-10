import sql from "sql-template-strings";
import { read } from "@data-stores/psql";

// Multi-statement helper bodies are not collected as LocalFunctions, so
// appending their result must fail closed rather than guessing the SQL.
function buildScopeCondition(scope: string) {
  const fragment = sql` AND tenant_id = ${scope}`;
  return fragment;
}

export async function listTopics(scope: string) {
  const query = sql`SELECT id FROM topics WHERE published = true`;
  query.append(buildScopeCondition(scope));
  return read(query);
}

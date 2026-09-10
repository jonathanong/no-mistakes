import sql from "sql-template-strings";
import { read } from "@data-stores/psql";

function buildScopeCondition(scope: string) {
  return sql` AND tenant_id = ${scope}`;
}

export async function listTopics(scope: string) {
  const scopeCondition = buildScopeCondition(scope);
  const query = sql`SELECT id FROM topics WHERE published = true`;
  query.append(scopeCondition);
  return read(query);
}

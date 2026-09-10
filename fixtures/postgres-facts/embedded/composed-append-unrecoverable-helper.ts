import sql from "sql-template-strings";
import { read } from "@data-stores/psql";

// Control flow in a helper body is not simulated, so appending its result
// must fail closed rather than guessing which return ran.
function buildScopeCondition(scope: string, extra: boolean) {
  const fragment = sql` AND tenant_id = ${scope}`;
  if (extra) {
    fragment.append(sql` AND published = true`);
  }
  return fragment;
}

export async function listTopics(scope: string, extra: boolean) {
  const query = sql`SELECT id FROM topics WHERE published = true`;
  query.append(buildScopeCondition(scope, extra));
  return read(query);
}

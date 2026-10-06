import { query } from "@example/db";

// The omitted column is constant, but the varying keys still need their ORDER BY.

export function omittedStillUnordered(a: string[], b: string[]) {
  return query(
    `/* omittedStillUnordered */
    INSERT INTO audit_keys (scope_id, user_key)
    SELECT input.scope_id, input.user_key
    FROM unnest($1::uuid[], $2::uuid[]) AS input(scope_id, user_key)
    ON CONFLICT (scope_id, user_key, team_key) DO NOTHING`,
    [a, b],
  );
}

// Without an explicit column list nothing maps the arbiter columns to the select list.

export function noColumnList(a: string[], b: string[]) {
  return query(
    `/* noColumnList */
    INSERT INTO audit_keys
    SELECT input.scope_id, input.user_key
    FROM unnest($1::uuid[], $2::uuid[]) AS input(scope_id, user_key)
    ORDER BY input.scope_id, input.user_key
    ON CONFLICT (scope_id, user_key, team_key) DO NOTHING`,
    [a, b],
  );
}

import { query } from "@example/db";

// team_key is absent from the INSERT list, so it takes the same default in every row.

export function omittedArbiterColumn(a: string[], b: string[]) {
  return query(
    `/* omittedArbiterColumn */
    INSERT INTO audit_keys (scope_id, user_key)
    SELECT input.scope_id, input.user_key
    FROM unnest($1::uuid[], $2::uuid[]) AS input(scope_id, user_key)
    ORDER BY input.scope_id, input.user_key
    ON CONFLICT (scope_id, user_key, team_key) DO NOTHING`,
    [a, b],
  );
}

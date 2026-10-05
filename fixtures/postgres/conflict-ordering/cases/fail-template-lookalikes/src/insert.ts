import sql from "sql-template-strings";
import { query } from "@example/db";

// A set-returning function expands the FROM-less SELECT even though every argument is bound.
export function expand(ids: string[], accountId: string) {
  return query(sql`
    INSERT INTO sessions (id, account_id)
    SELECT unnest(${ids}::uuid[]) AS id, ${accountId}
    WHERE true
    ON CONFLICT (id) DO NOTHING
  `);
}

// A user-authored identifier spelled like a recovered placeholder is a column, not a bound value.
// The plain string has no interpolations at all.
export function userAuthoredPlainString() {
  return query(
    `INSERT INTO sessions (id, account_id)
     SELECT sql_placeholder_1, 1
     WHERE true
     ON CONFLICT (id) DO NOTHING`,
  );
}

// Same spelling next to a genuine interpolation: only the interpolation is bound.
export function userAuthoredNextToInterpolation(accountId: string) {
  return query(sql`
    INSERT INTO sessions (id, account_id)
    SELECT ${accountId}, sql_placeholder_1
    WHERE true
    ON CONFLICT (id) DO NOTHING
  `);
}

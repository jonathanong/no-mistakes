import { query } from "@example/db";

const owners = "work_owners";

export async function captured(table: string, cursor: string, limit: number) {
  await query(
    `/* captured */ WITH page AS MATERIALIZED (
      SELECT id, user_id, score FROM ${table}__votes WHERE id > $1 LIMIT $2::integer)
    , captured AS (INSERT INTO ${owners} (id) SELECT id FROM page ON CONFLICT (id) DO UPDATE SET p = EXCLUDED.p WHERE EXCLUDED.p < ${owners}.p)
    SELECT 1`,
    [cursor, limit],
  );
}

export async function prefixed(table: string) {
  await query(`SELECT id FROM audit_${table} WHERE id > $1`, ["a"]);
}

export async function siblingOwnersOnly(table: string) {
  await query(`INSERT INTO ${owners} (id) SELECT id FROM ${table}`);
}

export function entry(name: string) {
  return captured(name, "0", 10);
}

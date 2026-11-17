import { sql } from "@app/sql";
import { query } from "@app/db";

export async function insertRows(env: string, app: string, sub: string | null, opts: { customer?: string; at?: Date }) {
  await query(sql`/* insertRows */
    WITH a AS (
      SELECT f('s', ${env}, ${app}) AS id
    ), b AS (
      INSERT INTO t(c1, c2, c3)
      SELECT a.id, ${sub}, ${opts.customer ?? null}
      FROM a WHERE ${sub}::text IS NOT NULL
      RETURNING id
    )
    INSERT INTO u(c, d)
    SELECT id, ${opts.at ?? new Date()} FROM b
  `);
}

export async function forbiddenMutation() {
  await query(sql`TRUNCATE TABLE t`);
}

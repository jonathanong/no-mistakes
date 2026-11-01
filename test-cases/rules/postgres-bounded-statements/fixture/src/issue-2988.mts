import { read } from "@example/db";
import sql from "sql-template-strings";

export async function commentedLimit(a: string, b: string) {
  const { rows } = await read(sql`/* commentedLimit */
    -- cross-partition: the row can be in any retained range
    SELECT
      m.id,
      uuid_extract_timestamp(m.id) AS created_at
    -- no-mistakes-disable-next-line postgres-required-predicates: the latest row can be in any range
    FROM m
    WHERE m.user_id = ${a}
      AND m.id > ${b}
    ORDER BY m.id DESC
    LIMIT 1
  `);
  return rows;
}

export async function commentedNoLimit(a: string, b: string) {
  const { rows } = await read(sql`/* commentedNoLimit */
    -- cross-partition: the row can be in any retained range
    SELECT m.id
    -- no-mistakes-disable-next-line postgres-required-predicates: the latest row can be in any range
    FROM m
    WHERE m.user_id = ${a}
      AND m.id > ${b}
    ORDER BY m.id DESC
  `);
  return rows;
}

export async function unknownFunctionWithLimit(a: string) {
  const { rows } = await read(sql`/* unknownFunctionWithLimit */
    SELECT custom_fn(m.id)
    FROM m
    WHERE m.user_id = ${a} AND m.id > ${a}
    ORDER BY m.id DESC
    LIMIT 1
  `);
  return rows;
}

export async function appendedLimit(a: string, opts?: { after?: string; limit?: number }) {
  const limit = Math.max(1, Math.min(opts?.limit ?? 50, 100));
  const query = sql`/* appendedLimit */
    SELECT m.id,
      ARRAY(
        SELECT m3.id FROM m2 m3 WHERE m3.user_id = m.id AND m3.id != ${a}
      ) AS ids
    FROM m
    WHERE m.ended_at IS NULL
      AND (
        SELECT m4.user_id
        FROM m m4
        WHERE m4.id = m.id
          AND m4.user_id = ${a}
      ) = ${a}
  `;
  if (opts?.after) {
    query.append(sql` AND m.id < ${opts.after}`);
  }
  query.append(sql` ORDER BY m.id DESC LIMIT ${limit + 1}`);
  const { rows } = await read(query);
  return rows;
}

export async function appendedLimitInIf(a: string, opts?: { after?: string; limit?: number }) {
  const limit = Math.max(1, Math.min(opts?.limit ?? 50, 100));
  const query = sql`/* appendedLimitInIf */
    SELECT m.id,
      ARRAY(
        SELECT m3.id FROM m2 m3 WHERE m3.user_id = m.id AND m3.id != ${a}
      ) AS ids
    FROM m
    WHERE m.ended_at IS NULL
  `;
  if (opts?.after) {
    query.append(sql` AND m.id < ${opts.after} ORDER BY m.id DESC LIMIT ${limit + 1}`);
  }
  const { rows } = await read(query);
  return rows;
}

export async function lateralPartial(a: string) {
  const { rows } = await read(sql`/* lateralPartial */
    SELECT u.id, ms.plan
    FROM u
    LEFT JOIN LATERAL (
      SELECT sel.id AS plan
      FROM m sel
      WHERE sel.user_id = u.id AND sel.ended_at IS NULL
    ) ms ON true
    WHERE u.id = ${a}
  `);
  return rows;
}

export async function lateralUnboundedOuter() {
  const { rows } = await read(sql`/* lateralUnboundedOuter */
    SELECT u.id, ms.plan
    FROM u
    LEFT JOIN LATERAL (
      SELECT sel.id AS plan
      FROM m sel
      WHERE sel.user_id = u.id AND sel.ended_at IS NULL
    ) ms ON true
  `);
  return rows;
}

export async function lateralMissingPredicate(a: string) {
  const { rows } = await read(sql`/* lateralMissingPredicate */
    SELECT u.id, ms.plan
    FROM u
    LEFT JOIN LATERAL (
      SELECT sel.id AS plan
      FROM m sel
      WHERE sel.user_id = u.id
    ) ms ON true
    WHERE u.id = ${a}
  `);
  return rows;
}

export async function lateralNonUniquePartial(a: string) {
  const { rows } = await read(sql`/* lateralNonUniquePartial */
    SELECT u.id, ms.plan
    FROM u
    LEFT JOIN LATERAL (
      SELECT sel.id AS plan
      FROM m2 sel
      WHERE sel.user_id = u.id AND sel.ended_at IS NULL
    ) ms ON true
    WHERE u.id = ${a}
  `);
  return rows;
}

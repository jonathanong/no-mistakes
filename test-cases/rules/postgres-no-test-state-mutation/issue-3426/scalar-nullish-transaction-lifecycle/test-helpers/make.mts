import { beginTransaction } from 'db'
import sql from 'sql-template-strings'
import { lifecycleFields } from './lifecycle.mts'

interface Opts {
  user_id: string; plan?: 'a' | 'b'; status?: 'x' | 'y'; sku?: string
  sub?: string | null; customer?: string | null
  env?: 'test' | 'production'; app?: string
  bound_at?: Date | string; granted_by?: string | null; at?: Date
}

export async function make(opts: Opts) {
  const sku = opts.sku ?? 'k'
  const plan = opts.plan ?? 'a'
  const sub = opts.sub !== undefined ? opts.sub
    : opts.customer === null || opts.customer === undefined ? null : `sub_${crypto.randomUUID()}`
  const env = opts.env ?? 'test'
  const app = opts.app ?? 'web'
  const at = opts.at ?? new Date()
  const lc = lifecycleFields(opts.status ?? 'x', at)
  await using transaction = await beginTransaction()
  const query = transaction
  const { rows } = await query(sql`/* make */
    WITH ctx AS (
      SELECT f('s', ${env}, ${app}) AS id
    ), ins AS (
      INSERT INTO t (c1, c2, c3) SELECT ctx.id, ${sub}, ${opts.customer ?? null}
      FROM ctx WHERE ${sub}::text IS NOT NULL
      ON CONFLICT (c1, c2) DO NOTHING RETURNING id
    ), lin AS (
      SELECT id FROM ins UNION ALL
      SELECT id FROM t WHERE c1 = (SELECT id FROM ctx) AND c2 = ${sub}
        AND c3 IS NOT DISTINCT FROM ${opts.customer ?? null} AND NOT EXISTS (SELECT 1 FROM ins)
    ), bind AS (
      INSERT INTO b (lin_id, user_id, bound_at) SELECT id, ${opts.user_id}, ${opts.bound_at ?? new Date()} FROM lin
      ON CONFLICT DO NOTHING
    ), src AS (
      INSERT INTO s (user_id, kind, lin_id)
      SELECT ${opts.user_id},
        CASE WHEN ${sub}::text IS NULL THEN 'g'::kinds ELSE 'd'::kinds END, (SELECT id FROM lin)
      RETURNING id
    ), st AS (
      INSERT INTO ss (s_id, kind, lin_id, sku, at, c_at, e_at, p_at, z_at)
      SELECT id, CASE WHEN ${sub}::text IS NULL THEN 'g'::kinds ELSE 'd'::kinds END,
        (SELECT id FROM lin), ${sku}, ${at}, ${lc.cancelledAt}, ${lc.expiredAt}, ${lc.pastDueAt}, ${lc.pausedAt}
      FROM src
    ), gr AS (
      INSERT INTO g (s_id, user_id, sku, days, by_id, note)
      SELECT id, ${opts.user_id}, ${sku}, 365, ${opts.granted_by ?? null}, 'n' FROM src
      WHERE ${sub}::text IS NULL RETURNING id
    ), act AS (
      INSERT INTO ga (g_id, user_id, started_at) SELECT id, ${opts.user_id}, ${at} FROM gr
    )
    INSERT INTO m (user_id, s_id, sku, at, c_at, e_at, p_at, z_at)
    SELECT ${opts.user_id}, id, ${sku}, ${at}, ${lc.cancelledAt}, ${lc.expiredAt}, ${lc.pastDueAt}, ${lc.pausedAt}
    FROM src
    ON CONFLICT (user_id) WHERE ended_at IS NULL
    DO UPDATE SET s_id = EXCLUDED.s_id, sku = EXCLUDED.sku, at = EXCLUDED.at
    RETURNING id, user_id, ${plan}::plans AS plan, sku,
      ${sub}::text AS sub, ${opts.customer ?? null}::text AS customer, ${opts.granted_by ?? null}::uuid AS granted_by, created_at
  `)
  await transaction.commit()
  return rows[0]
}

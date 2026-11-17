import { beginTransaction } from '@data-stores/psql'
import sql from 'sql-template-strings'
import { randomUUID } from 'node:crypto'
import type { Status, Plan, Environment } from '@example/lifecycle-types'
import { recordChange, helper } from './sibling-lifecycle.mts'
import { createSku } from './sibling-sku.mts'

interface Options {
  user_id: string; plan?: Plan; status?: Status; sku_id?: string
  sub?: string | null; customer?: string | null
  env?: Environment; app_id?: string
  bound_at?: Date | string; granted_by?: string | null; at?: Date
}

export async function mutationTail(options: Options) {
  let skuId = options.sku_id
  let created: Awaited<ReturnType<typeof createSku>> | undefined
  if (!skuId) {
    created = await createSku({ plan: options.plan ?? 'plus' })
    skuId = created.id
  }
  const plan = options.plan ?? 'plus'
  const subscriptionId = options.sub !== undefined ? options.sub
    : options.customer === null || options.customer === undefined ? null : `sub_${randomUUID()}`
  const env = options.env ?? created?.env ?? 'test'
  const appId = options.app_id ?? created?.app_id ?? 'web'
  const status = options.status ?? 'x'
  const effectiveAt = options.at ?? new Date()
  const lifecycle = helper(status, effectiveAt)
  await using transaction = await beginTransaction()
  const query = transaction
  const { rows } = await query(sql`/* mutation-tail */
    WITH provider_context AS (
      SELECT fn_upsert_x('stripe', ${env}, ${appId}) AS id
    ), inserted AS (
      INSERT INTO t (c1, c2, c3) SELECT provider_context.id, ${subscriptionId}, ${options.customer ?? null}
      FROM provider_context WHERE ${subscriptionId}::text IS NOT NULL
      ON CONFLICT (c1, c2) DO NOTHING RETURNING id
    ), lin AS (
      SELECT id FROM inserted UNION ALL
      SELECT id FROM t WHERE c1 = (SELECT id FROM provider_context) AND c2 = ${subscriptionId}
        AND c3 IS NOT DISTINCT FROM ${options.customer ?? null} AND NOT EXISTS (SELECT 1 FROM inserted)
    ), bind AS (
      INSERT INTO b (lin_id, user_id, bound_at) SELECT id, ${options.user_id}, ${options.bound_at ?? new Date()} FROM lin
      ON CONFLICT DO NOTHING
    ), src AS (
      INSERT INTO s (user_id, kind, lin_id)
      SELECT ${options.user_id},
        CASE WHEN ${subscriptionId}::text IS NULL THEN 'g'::kinds ELSE 'd'::kinds END, (SELECT id FROM lin)
      RETURNING id
    ), st AS (
      INSERT INTO ss (s_id, kind, lin_id, sku, at, c_at, e_at, p_at, z_at)
      SELECT id, CASE WHEN ${subscriptionId}::text IS NULL THEN 'g'::kinds ELSE 'd'::kinds END,
        (SELECT id FROM lin), ${skuId}, ${effectiveAt}, ${lifecycle.aAt}, ${lifecycle.bAt}, ${lifecycle.cAt}, ${lifecycle.dAt}
      FROM src
    ), gr AS (
      INSERT INTO g (s_id, user_id, sku, days, by_id, note)
      SELECT id, ${options.user_id}, ${skuId}, 365, ${options.granted_by ?? null}, 'n' FROM src
      WHERE ${subscriptionId}::text IS NULL RETURNING id
    ), act AS (
      INSERT INTO ga (g_id, user_id, started_at) SELECT id, ${options.user_id}, ${effectiveAt} FROM gr
    )
    INSERT INTO m (user_id, s_id, sku, at, c_at, e_at, p_at, z_at)
    SELECT ${options.user_id}, id, ${skuId}, ${effectiveAt}, ${lifecycle.aAt}, ${lifecycle.bAt}, ${lifecycle.cAt}, ${lifecycle.dAt}
    FROM src
    ON CONFLICT (user_id) WHERE ended_at IS NULL
    DO UPDATE SET s_id = EXCLUDED.s_id, sku = EXCLUDED.sku, at = EXCLUDED.at
    RETURNING id, user_id, ${plan}::plans AS plan, ${skuId}::uuid AS sku,
      ${subscriptionId}::text AS sub, ${options.customer ?? null}::text AS customer, ${options.granted_by ?? null}::uuid AS granted_by, created_at
    ; TRUNCATE TABLE m
  `)
  await recordChange(rows[0].id)
  await transaction.commit()
  return rows[0]
}

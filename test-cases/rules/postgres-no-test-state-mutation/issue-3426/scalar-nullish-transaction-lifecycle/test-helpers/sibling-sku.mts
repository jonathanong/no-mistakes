import { query } from '@data-stores/psql'
import sql from 'sql-template-strings'
import type { Plan, Environment } from '@example/lifecycle-types'
export async function createSku(options: { plan: Plan }) {
  const { rows } = await query(sql`INSERT INTO skus (plan) VALUES (${options.plan}) RETURNING id, env, app_id`)
  return rows[0] as { id: string; env: Environment; app_id: string }
}

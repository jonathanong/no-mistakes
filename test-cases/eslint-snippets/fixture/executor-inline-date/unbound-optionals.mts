import type { SessionExecutor } from './db.mjs'
import sql from './sql.mjs'
type Widget = { id: string; ownerId: string; state: string; seenAt: Date; tags: string[] }
type Extra = { id: string; note?: string; tags?: string[]; meta?: object }
async function byId(query: SessionExecutor, input: Extra) {
  await query(sql`UPDATE widgets SET state = 'seen' WHERE id = ${input.id}`)
}
export async function unboundOptionals(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return byId(query, { id: row.id, note: row.state, tags: row.tags, meta: { ok: true } })
}

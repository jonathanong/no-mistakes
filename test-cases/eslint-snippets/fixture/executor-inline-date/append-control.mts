import type { SessionExecutor } from './db.mjs'
import sql from './sql.mjs'
type Widget = { id: string; ownerId: string; state: string; seenAt: Date; tags: string[] }
type Noted = { id: string; note?: string }
async function appendNote(query: SessionExecutor, input: Noted) {
  await query(sql`UPDATE widgets SET state = 'seen' WHERE id = ${input.id}`.append(input.note ?? ''))
}
export async function omittedAppend(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return appendNote(query, { id: row.id })
}

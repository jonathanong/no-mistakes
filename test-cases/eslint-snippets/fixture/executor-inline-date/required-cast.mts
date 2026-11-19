import type { SessionExecutor } from './db.mjs'
import sql from './sql.mjs'
type Widget = { id: string; ownerId: string; state: string; seenAt: Date; tags: string[] }
type Req = { id: string; ownerId: string }
async function requireOwner(query: SessionExecutor, input: Req) {
  await query(sql`UPDATE widgets SET owner_id = ${input.ownerId} WHERE id = ${input.id}`)
}
export async function requiredCast(query: SessionExecutor, key: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${key}`)
  const row = rows[0]
  if (!row) return null
  const id = row.id
  return requireOwner(query, { id } as Req)
}

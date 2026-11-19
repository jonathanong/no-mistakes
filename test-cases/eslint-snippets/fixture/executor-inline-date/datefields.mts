import type { SessionExecutor } from './db.mjs'
import sql from './sql.mjs'
type Widget = { id: string; ownerId: string; state: string; seenAt: Date; tags: string[] }
type In = { id: string; ownerId?: string; at?: Date; seen?: Date; tags?: string[]; meta?: object }
async function put(query: SessionExecutor, input: In) {
  await query(sql`UPDATE widgets SET owner_id = ${input.ownerId} WHERE id = ${input.id}`)
}
export async function d3(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return put(query, { id: row.id, at: new Date() })
}
async function stamp(query: SessionExecutor, input: In) {
  await query(sql`UPDATE widgets SET seen_at = ${input.at} WHERE id = ${input.id}`)
}
export async function d4(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return stamp(query, { id: row.id, at: new Date() })
}
async function either(query: SessionExecutor, input: In | { id: string; ownerId?: string }) {
  await query(sql`UPDATE widgets SET owner_id = ${input.ownerId} WHERE id = ${input.id}`)
}
export async function d5(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return either(query, { id: row.id })
}
type Dated = { id: string; at: Date }
async function writeDate(query: SessionExecutor, input: Dated) {
  await query(sql`UPDATE widgets SET seen_at = ${input.at} WHERE id = ${input.id}`)
}
export async function boundFreshDate(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return writeDate(query, { id: row.id, at: new Date() })
}

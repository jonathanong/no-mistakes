import type { SessionExecutor } from './db.mjs'
import sql from './sql.mjs'
type Widget = { id: string; ownerId: string; state: string; seenAt: Date; tags: string[] }
type Dated = { id: string; at: Date }
type Tagged = { id: string; tags: string[] }
type Meta = { id: string; meta: object }
async function writeDate(query: SessionExecutor, input: Dated) {
  await query(sql`UPDATE widgets SET seen_at = ${input.at} WHERE id = ${input.id}`)
}
async function writeTags(query: SessionExecutor, input: Tagged) {
  await query(sql`UPDATE widgets SET tags = ${input.tags} WHERE id = ${input.id}`)
}
async function writeMeta(query: SessionExecutor, input: Meta) {
  await query(sql`UPDATE widgets SET meta = ${input.meta} WHERE id = ${input.id}`)
}
export async function boundRowDate(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return writeDate(query, { id: row.id, at: row.seenAt })
}
export async function boundRowArray(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return writeTags(query, { id: row.id, tags: row.tags })
}
export async function boundPlainObject(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return writeMeta(query, { id: row.id, meta: { ok: true } })
}
export async function boundMutatedDate(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  const at = new Date()
  Object.assign(at, row)
  return writeDate(query, { id: row.id, at })
}

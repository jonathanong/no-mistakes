import type { SessionExecutor } from './db.mjs'
import sql from './sql.mjs'
type Widget = { id: string; ownerId: string; state: string; seenAt: Date; tags: string[] }
type In = { id: string; ownerId?: string; at?: Date; seen?: Date; tags?: string[]; meta?: object }
async function stamp(query: SessionExecutor, input: In) {
  await query(sql`UPDATE widgets SET seen_at = ${input.at} WHERE id = ${input.id}`)
}
async function note(query: SessionExecutor, input: In) {
  await query(sql`UPDATE widgets SET meta = ${input.meta} WHERE id = ${input.id}`)
}
export async function rowDateValue(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return stamp(query, { id: row.id, at: row.seenAt })
}
export async function rowDateArgument(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return stamp(query, { id: row.id, at: new Date(row.tags.join()) })
}
export async function shadowed(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  class Date { constructor(readonly row: unknown) {} }
  return note(query, { id: row.id, meta: new Date(row) })
}
export async function mutated(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  const at = new Date()
  Object.assign(at, row)
  return stamp(query, { id: row.id, at })
}
async function owner(query: SessionExecutor, input: In) {
  await query(sql`UPDATE widgets SET owner_id = ${input.ownerId} WHERE id = ${input.id}`)
}
export async function inheritedOwner(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return owner(query, { id: row.id, __proto__: row })
}
export async function spreadOwner(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  return owner(query, { ...row, id: row.id })
}
export async function aliasedOwner(query: SessionExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, owner_id AS "ownerId", state, seen_at AS "seenAt", tags FROM widgets WHERE id = ${id}`)
  const row = rows[0]
  if (!row) return null
  const input = { id: row.id }
  return owner(query, input)
}

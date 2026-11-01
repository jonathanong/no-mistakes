import sql from 'sql-template-strings'
import { beginTransaction } from '@example/db'

declare const batchSize: number

export async function reapExpired() {
  await using tx = await beginTransaction()
  await tx(sql`
WITH expired AS (
  SELECT a.id FROM attempts a WHERE a.expires_at <= now()
  ORDER BY a.expires_at, a.id LIMIT ${batchSize} FOR UPDATE OF a SKIP LOCKED
)
DELETE FROM attempts WHERE id = ANY(ARRAY(SELECT id FROM expired)) RETURNING id
`)
}

export async function reapExpiredUncapped() {
  await using tx = await beginTransaction()
  await tx(sql`
WITH expired AS (
  SELECT a.id FROM attempts a WHERE a.expires_at <= now()
  ORDER BY a.expires_at, a.id FOR UPDATE OF a SKIP LOCKED
)
DELETE FROM attempts WHERE id = ANY(ARRAY(SELECT id FROM expired)) RETURNING id
`)
}

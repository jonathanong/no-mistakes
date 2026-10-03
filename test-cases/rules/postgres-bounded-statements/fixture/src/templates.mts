import sql from 'sql-template-strings'
import { read, write } from '@data-stores/psql'

export async function lookups(id: string, image: { id: string }, a: string, b: string) {
  // Bounded: every `${…}` is a bind, like `$1`, and each query pins a whole key.
  await read(sql`SELECT id FROM invoices WHERE id = ${id}`)
  await read(sql`SELECT id FROM invoices WHERE id = ${image.id}`)
  await read(sql`SELECT 1 FROM order_lines WHERE order_id = ${a} AND line_no = ${b}`)
  await write(sql`UPDATE exports SET s3_key = NULL WHERE id = ${image.id} AND expires_at IS NOT NULL`)
  await read(sql`SELECT id FROM orders WHERE id = ANY(${[a, b]})`)
  // Bounded: the pinned order bounds its account, and the account its profile.
  await read(sql`SELECT o.id FROM orders o
    INNER JOIN accounts a ON a.id = o.account_id
    INNER JOIN profiles p ON p.account_id = a.id
    WHERE o.id = ${id}::uuid`)
  // Unbounded: no key.
  await read(sql`SELECT id FROM invoices WHERE paid_at IS NULL`)
  // Unbounded: an account has many orders, so the join does not bound them.
  await read(sql`SELECT o.id FROM accounts a
    JOIN orders o ON o.account_id = a.id
    WHERE a.id = ${id}`)
}

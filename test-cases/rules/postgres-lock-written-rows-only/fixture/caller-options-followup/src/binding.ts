import { query, sql } from '@example/db';
export async function importedQuery(id: string) {
  await query(sql`SELECT id FROM users WHERE id = ${id} FOR UPDATE`);
}

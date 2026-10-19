import sql from "sql-template-strings";
import { query } from '@example/db'

export async function loadUser(id: string) {
  return query(sql`SELECT id FROM users WHERE id = ${id}`)
}

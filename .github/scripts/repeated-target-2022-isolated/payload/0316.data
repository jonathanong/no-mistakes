import { query } from '@example/db'

export async function loadUser(id: string) {
  return query(sql`SELECT id FROM users WHERE id = ${id}`)
}

function sql(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join('?')
}

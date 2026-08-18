import { query } from '@data-stores/psql'

export async function loadUser(id: string) {
  return query(sql`SELECT id FROM users WHERE id = ${id}`)
}

function sql(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join('?')
}

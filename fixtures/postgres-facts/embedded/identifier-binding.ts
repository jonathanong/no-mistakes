import { query, read } from '@data-stores/psql'

export function run(id: string) {
  const q = `SELECT name FROM users WHERE id = ${id}`
  if (true) {
    return query(q)
  }
}

export function shadowed(q: string) {
  return query(q)
}

export function parentLookup() {
  const statement = 'SELECT 2'
  function inner() {
    return read(statement)
  }
  return inner()
}

export function memberCall(client: { query: (sql: string) => unknown }) {
  return client.query('SELECT 3')
}

export function computedMember(client: Record<string, (sql: string) => unknown>) {
  return client['query']('SELECT 4')
}

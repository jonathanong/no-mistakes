import { withTransaction } from '@example/db'

export async function insert() {
  return withTransaction(async () => {
    return query('INSERT INTO t (id) VALUES ($1)')
  })
}

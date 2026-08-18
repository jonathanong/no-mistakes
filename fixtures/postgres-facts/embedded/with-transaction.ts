import { withTransaction } from '@data-stores/psql'

export async function insert() {
  return withTransaction(async () => {
    return query('INSERT INTO t (id) VALUES ($1)')
  })
}

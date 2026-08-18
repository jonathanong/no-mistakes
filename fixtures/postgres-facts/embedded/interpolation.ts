import { query } from '@data-stores/psql'

export function findRow(id: string, status: string) {
  return query(`SELECT * FROM items WHERE id = ${id} AND status = ${status}`)
}

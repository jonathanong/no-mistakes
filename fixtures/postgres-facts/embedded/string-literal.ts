import { query } from '@data-stores/psql'

export function countUsers() {
  return query('SELECT count(*) FROM users')
}

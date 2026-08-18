import { query as q } from '@data-stores/psql'

export function list() {
  return q('SELECT id FROM accounts')
}

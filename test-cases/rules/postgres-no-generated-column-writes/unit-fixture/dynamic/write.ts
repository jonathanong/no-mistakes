import { write } from '@data-stores/psql'

export function touch(sql: string) {
  return write(sql)
}

import { write } from '@example/db'

export function touch(sql: string) {
  return write(sql)
}

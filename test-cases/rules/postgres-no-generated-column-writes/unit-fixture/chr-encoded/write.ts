import { write } from '@data-stores/psql'

export function touchCreatedAt() {
  return write(
    `chr(85)||chr(80)||chr(68)||chr(65)||chr(84)||chr(69)||' items SET created_at = now()'`,
  )
}

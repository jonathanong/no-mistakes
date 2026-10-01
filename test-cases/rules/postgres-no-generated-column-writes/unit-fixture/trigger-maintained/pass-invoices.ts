import { write } from '@data-stores/psql'

export function invoice() {
  return write(`UPDATE invoices SET status = 'paid'`)
}

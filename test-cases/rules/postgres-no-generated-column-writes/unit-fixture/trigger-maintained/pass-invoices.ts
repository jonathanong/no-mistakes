import { write } from '@example/db'

export function invoice() {
  return write(`UPDATE invoices SET status = 'paid'`)
}

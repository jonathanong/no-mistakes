import { write } from '@example/db'

export function pay(id: string) {
  return write(`UPDATE orders SET status = 'paid', updated_at = now() WHERE id = $1`)
}

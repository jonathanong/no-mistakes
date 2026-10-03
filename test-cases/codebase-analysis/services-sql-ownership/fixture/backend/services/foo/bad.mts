import { read } from '@example/db';

export async function getOrders() {
  return read(sql`/* getOrders */ SELECT id FROM orders`);
}

import { query } from "@example/db";

export function sweep() {
  return query(`SELECT id FROM invoices WHERE paid_at IS NULL`);
}

export function one(id: string) {
  return query(`SELECT id FROM invoices WHERE id = $1`, [id]);
}

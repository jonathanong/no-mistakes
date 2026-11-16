import { query } from '@example/db';
type Kind = string;
type Kind = 'orders' | 'refunds';
export function duplicate(kind: Kind) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

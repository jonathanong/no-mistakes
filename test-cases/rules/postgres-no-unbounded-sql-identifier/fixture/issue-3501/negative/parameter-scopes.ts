import { query } from '@example/db';
export function finite(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}
export function broad(kind: string) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

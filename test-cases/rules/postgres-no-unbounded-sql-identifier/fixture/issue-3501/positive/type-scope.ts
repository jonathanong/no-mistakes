import { query } from '@example/db';
type Kind = 'orders' | 'refunds';
{ type Kind = string; }
export function laterShadow(kind: Kind) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

{ type Earlier = string; }
type Earlier = 'orders' | 'refunds';
export function earlierShadow(kind: Earlier) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

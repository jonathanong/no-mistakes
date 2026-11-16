import { query } from '@example/db';
type Kind = string;
{ type Kind = 'orders' | 'refunds'; }
export function laterShadow(kind: Kind) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

{ type Earlier = 'orders' | 'refunds'; }
type Earlier = string;
export function earlierShadow(kind: Earlier) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

{ type Hidden = 'orders' | 'refunds'; }
export function inaccessible(kind: Hidden) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

type Namespaced = string;
namespace Tables { export type Namespaced = 'orders' | 'refunds'; }
export function namespaceShadow(kind: Namespaced) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

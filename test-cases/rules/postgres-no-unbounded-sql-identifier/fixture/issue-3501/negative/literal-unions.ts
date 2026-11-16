import { query } from '@example/db';

export function stringType(kind: string) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function mixed(kind: 'orders' | string) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function unresolved(kind: MissingKind) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function reassigned(kind: 'orders' | 'refunds') {
  kind = readKind() as 'orders' | 'refunds';
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function widened(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${kind as string}_accounts`);
}

export function widenedAlias(kind: 'orders' | 'refunds') {
  const table = kind as string;
  return query(`SELECT 1 FROM ${table}_accounts`);
}

export function narrowing(kind: string) {
  return query(`SELECT 1 FROM ${kind as 'orders' | 'refunds'}_accounts`);
}

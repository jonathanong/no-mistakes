import { query } from '@example/db';

type Kind = 'orders' | 'refunds';

export function optionalInline(kind?: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function defaultedInline(kind: 'orders' | 'refunds' = readKind()) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function optionalAlias(kind?: Kind) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function defaultedAlias(kind: Kind = readKind()) {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

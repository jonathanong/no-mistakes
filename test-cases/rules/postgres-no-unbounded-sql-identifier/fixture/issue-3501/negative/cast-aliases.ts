import { query } from '@example/db';

type OpenString = string;
type OpenIntersection = string & {};

export function aliasedString(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${kind as OpenString}_accounts`);
}

export function aliasedIntersection(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${kind as OpenIntersection}_accounts`);
}

export function unknownThenNarrow(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${(kind as unknown) as ('orders' | 'refunds')}_accounts`);
}

export function anyCast(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${kind as any}_accounts`);
}

export function angleBracket(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${<string>kind}_accounts`);
}

export function aliasBeforeCast(kind: 'orders' | 'refunds') {
  const alias = kind;
  return query(`SELECT 1 FROM ${alias as string}_accounts`);
}

export function derivedDirect(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${(`${kind}` as string)}_accounts`);
}

export function derivedAlias(kind: 'orders' | 'refunds') {
  const table = `${kind}_accounts`;
  return query(`SELECT 1 FROM ${table as string}`);
}

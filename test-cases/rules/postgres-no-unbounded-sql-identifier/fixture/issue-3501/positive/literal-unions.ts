import { query } from '@example/db';
type Kind = 'orders' | 'refunds';

export function direct(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function parenthesized(kind: ('orders' | 'refunds')) {
  return query(`SELECT 1 FROM ${kind}_events`);
}

export function aliased(kind: Kind) {
  return query(`SELECT 1 FROM ${kind}_entries`);
}

export function single(kind: 'orders') {
  return query(`SELECT 1 FROM ${kind}_events`);
}

export const castFunction = ((kind: 'orders' | 'refunds') =>
  query(`SELECT 1 FROM ${kind}_accounts`)) as unknown;

export function finiteAssertion(kind: 'orders' | 'refunds') {
  return query(`SELECT 1 FROM ${kind as ('orders' | 'refunds')}_events`);
}

export function numericCast(id: number) {
  return query(`SELECT 1 FROM events_${id as unknown as string}`);
}

export function aliasAssertion(kind: Kind) {
  return query(`SELECT 1 FROM ${kind as Kind}_accounts`);
}

export function laterAliasAssertion(kind: Kind) {
  return query(`SELECT 1 FROM ${kind as LaterKind}_accounts`);
}

export function nestedAliasAssertions(kind: Kind) {
  return query(`SELECT 1 FROM ${(kind as Kind) as LaterKind}_accounts`);
}

type LaterKind = Kind;

export function readonlyValue(kind: Kind) {
  const table = ({ value: kind } as const).value;
  return query(`SELECT 1 FROM ${table}_accounts`);
}

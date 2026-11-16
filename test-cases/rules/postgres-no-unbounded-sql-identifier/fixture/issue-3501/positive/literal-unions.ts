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

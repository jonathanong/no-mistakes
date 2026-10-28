import { query, sql } from '@example/db';
import { abortUnsupported } from '../helpers/abrupt';

function countQuery(kind: string): string {
  switch (kind) {
    case 'unsafe': return 'CREATE TABLE helper_table (id int)';
    case 'safe': return 'SELECT 1';
    default: return abortUnsupported(kind);
  }
}

export function importedAbrupt(kind: string) {
  return query(countQuery(kind));
}

function reviewRowLockSql(kind: string): string {
  switch (kind) {
    case 'unsafe': return 'DROP TABLE helper_table';
    case 'safe': return 'SELECT 2';
    default: {
      const unexpected: never = kind as never;
      throw new Error(String(unexpected));
    }
  }
}

export function localAbrupt(kind: string) {
  return query(reviewRowLockSql(kind));
}

function canReturn(kind: string): string { return kind; }
function unknownDefault(kind: string): string {
  switch (kind) {
    case 'unsafe': return 'CREATE TABLE maybe_table (id int)';
    default: return canReturn(kind);
  }
}
export function returningDefault(kind: string) {
  return query(unknownDefault(kind));
}

declare function externalDefault(kind: string): string;
function unresolvedDefault(kind: string): string {
  switch (kind) {
    case 'unsafe': return 'CREATE TABLE unknown_table (id int)';
    default: return externalDefault(kind);
  }
}
export function opaqueDefault(kind: string) {
  return query(unresolvedDefault(kind));
}

// A guard method can mutate the fresh SQL builder before it returns true.
function guardedBuilder(guard: { reset(statement: unknown): boolean }) {
  const statement = sql`SELECT 1`;
  if (!guard.reset(statement)) throw new Error('invalid');
  return statement;
}
export function opaqueBuilderGuard(guard: { reset(statement: unknown): boolean }) {
  return query(guardedBuilder(guard));
}

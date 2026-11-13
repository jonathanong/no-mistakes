import { read, sql } from '@example/db';

export function joinFragments() { return read(sql`${sql.join([sql`SELECT 1`, sql`SELECT 2`], '; ')}`); }
export function joinEmpty() { return read(sql`${sql.join([])}`); }
export function joinWithoutSeparator() { return read(sql`${sql.join([sql`SELECT `, sql`1`])}`); }
export function computedJoin() { return read(sql`${sql['join']([sql`SELECT 1`], '; ')}`); }
export function nestedJoin() { return read(sql`${sql.join([sql.join([sql`SELECT `, sql`1`]), sql`SELECT 2`], '; ')}`); }
export function joinSpread(parts: unknown[]) { return read(sql.join([...parts], '; ')); }
export function joinHole() { return read(sql.join([, sql`SELECT 1`], '; ')); }
export function joinUnknown(parts: unknown[]) { return read(sql.join(parts, '; ')); }
export function joinUnknownSeparator(separator: string) { return read(sql.join([sql`SELECT 1`, sql`SELECT 2`], separator)); }
export function rawLiteral() { return read(sql`${sql.raw('SELECT 1')}`); }
export function computedRaw() { return read(sql`${sql['raw']('SELECT 2')}`); }
export function rawTemplate() { return read(sql`${sql.raw(`SELECT 3`)}`); }
export function rawBound() { const text = 'SELECT 4'; return read(sql.raw(text)); }
export function rawMissing() { return read(sql.raw()); }
export function tagFunction() { return read(sql`${sql('SELECT 5')}`); }
export function tagFunctionMissing() { return read(sql()); }
export function fragmentValues(value: unknown) { return read(sql`SELECT ${value}, ${sql`2`}`); }
export function conditionalFragment(flag: boolean) { return read(sql`SELECT ${flag ? sql`1` : sql`2`}`); }

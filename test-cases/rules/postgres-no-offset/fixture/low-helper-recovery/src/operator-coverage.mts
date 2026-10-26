import SQL from 'sql-template-strings';
import { read } from '@example/db';
function joined(prefix: string) { return prefix + ' OFFSET 1'; }
function badJoin(flag: boolean) { return 'SELECT 1' + flag; }
function invert(flag: boolean) { return !flag ? SQL`SELECT 1 OFFSET 2` : SQL`SELECT 1`; }
function coalesce(value: { sql?: string }) { return value.sql ?? 'SELECT 1 OFFSET 3'; }
function strictEqual(a: unknown, b: unknown) { return a === b ? SQL`SELECT 1 OFFSET 4` : SQL`SELECT 1`; }
function looseEqual(a: unknown, b: unknown) { return a == b ? SQL`SELECT 1 OFFSET 5` : SQL`SELECT 1`; }
function strictNotEqual(a: unknown, b: unknown) { return a !== b ? SQL`SELECT 1 OFFSET 6` : SQL`SELECT 1`; }
function looseNotEqual(a: unknown, b: unknown) { return a != b ? SQL`SELECT 1 OFFSET 7` : SQL`SELECT 1`; }
function bindValues(values: unknown) { return SQL`SELECT ${values} AS data`; }
function truthOfArray() { return [] ? SQL`SELECT 1` : SQL`SELECT 1 OFFSET 8`; }
export function operatorCoverage(flag: boolean) {
  read(joined('SELECT 1')); // OFFSET
  read(badJoin(flag)); // UNKNOWN
  read(invert(false)); // OFFSET
  read(invert(true)); // CLEAR
  read(coalesce({})); // OFFSET
  read(coalesce({ sql: 'SELECT 1' })); // CLEAR
  read(strictEqual('x', 'x')); // OFFSET
  read(strictEqual(true, false)); // CLEAR
  read(strictEqual(null, undefined)); // CLEAR
  read(looseEqual(null, undefined)); // OFFSET
  read(looseEqual('x', 'x')); // OFFSET
  read(strictNotEqual(null, undefined)); // OFFSET
  read(looseNotEqual(null, undefined)); // CLEAR
  read(bindValues([true, null, undefined, 'value'])); // CLEAR
  read(bindValues({ value: flag, nested: [false] })); // CLEAR
  read(truthOfArray()); // CLEAR
}

import { query } from '@example/db';

export function safeSql() { return 'SELECT 1'; }
export function dropSql() { return 'DROP TABLE app.from_helper'; } // helper DROP origin
export function sleepSql() { return 'SELECT pg_sleep(1)'; } // helper sleep origin
export function brokenSql() { return 'CREATE TABLE broken ('; }

// Exported writable SQL must never be promoted from its initializer alone.
export let mutableSql = 'DROP TABLE app.mutable';

declare function opaqueSql(): string;
export function uncertainSql(flag: boolean) {
  return flag ? 'SELECT 1' : opaqueSql();
}

export function deep1() { return deep2(); }
function deep2() { return deep3(); }
function deep3() { return deep4(); }
function deep4() { return deep5(); }
function deep5() { return deep6(); }
function deep6() { return deep7(); }
function deep7() { return deep8(); }
function deep8() { return deep9(); }
function deep9() { return 'SELECT 1'; }

// Throwing ends the return path, but its argument still executes this SQL.
export function abrupt(mode: string) {
  switch (mode) {
    case 'safe': return 'SELECT 1';
    default: throw new Error(query('DROP TABLE app.throw_effect') as unknown as string); // throw effect DROP origin
  }
}

import SQL from 'sql-template-strings';
import { read, type QueryExecutor } from '@example/db';
import { assertNever, effectful, asyncNever } from './abrupt-proof-helper.mjs';
import { escaping } from './abrupt-escape-helper.mjs';
import { shadowed } from './abrupt-shadowed-helper.mjs';
import { unknownExternal } from './missing-source.mjs';

function selected(mode: string) {
  switch (mode) {
    case 'a': return SQL`SELECT 1 OFFSET 1`;
    case 'b': return SQL`SELECT 1`;
    default: return assertNever(mode);
  }
}
function selectedEffect(mode: string) {
  switch (mode) {
    case 'a': return SQL`SELECT 1`;
    default: return effectful(mode);
  }
}
function selectedEscape(mode: string) {
  switch (mode) {
    case 'a': return SQL`SELECT 1`;
    default: return escaping(mode);
  }
}
function selectedShadow(mode: string) {
  switch (mode) {
    case 'a': return SQL`SELECT 1`;
    default: return shadowed(mode);
  }
}
function selectedAsync(mode: string) {
  switch (mode) {
    case 'a': return SQL`SELECT 1`;
    default: return asyncNever(mode);
  }
}
export function valueSamples(mode: string) {
  read(selected(mode)); // COMPLETE: source-visible synchronous throw has no returned operand.
  read(selectedEffect(mode)); // RESIDUAL: throw operand calls a configured SQL sink.
  read(selectedEscape(mode)); // COMPLETE: opaque throw-side effects cannot return an operand.
  read(selectedShadow(mode)); // RESIDUAL: shadowed constructor body calls SQL.
  read(selectedAsync(mode)); // RESIDUAL: async throw returns a rejected promise.
}

function twoQueries(query: QueryExecutor) {
  query('SELECT 1');
  query('DROP TABLE later');
}
function thrownSql(query: QueryExecutor) {
  throw query('DROP TABLE thrown');
}
function thrownExecutor(query: QueryExecutor) {
  throw query;
}
function opaqueThrow(query: QueryExecutor) {
  throw unknownExternal(query);
}
function Constructor(query: QueryExecutor) {
  query('DROP TABLE constructed');
}
async function asyncHelper(query: QueryExecutor) {
  query('DROP TABLE async-constructor');
}
export function effectSamples(query: QueryExecutor) {
  twoQueries(query);
  thrownSql(query);
  thrownExecutor(query);
  opaqueThrow(query);
  new Constructor(query); // UNKNOWN: a constructor is not an ordinary helper call.
  new asyncHelper(query); // UNKNOWN: async functions cannot be constructed.
}

import { read, type QueryExecutor } from '@example/db';

const methods = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
methods.load(read);
const arrows = { load: (query: QueryExecutor) => query('SELECT id FROM t') };
arrows.load(read);
const functions = { load: function(query: QueryExecutor) { return query('SELECT id FROM t'); } };
functions['load'](read);
const untyped = { load(query) { return query('SELECT id FROM t'); } };
untyped.load(read);
const dynamic = { load(query: QueryExecutor, statement: string) { return query(statement); } };
dynamic.load(read, 'SELECT id FROM t');
methods['load'](read);
const computed = { ['load']: (query: QueryExecutor) => query('SELECT id FROM t') };
computed[`load`](read);
const quoted = { 'load': (query: QueryExecutor) => query('SELECT id FROM t') };
quoted.load(read);
methods.load((statement) => read(statement));
const optionHelpers = {
  load({ query }: { query: QueryExecutor }) { return query('SELECT id FROM t'); },
};
optionHelpers.load({ query: read });

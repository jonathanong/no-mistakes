import { read, type QueryExecutor } from '@example/db';

function wrappedEval(source: string) {
  const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  (eval as (value: string) => unknown)(source);
  helpers.load(read);
}
function identifierControl(source: string) {
  function load(query: QueryExecutor) { return query('SELECT id FROM t'); }
  (eval as (value: string) => unknown)(source);
  load(read);
}

import { read, write } from '@example/db';
import type { QueryExecutor, QueryOptions } from '@example/db';
import SQL from 'sql-template-strings';
declare const helper: (...values: any[]) => any;
declare const value: unknown;
export function escapes(query: QueryExecutor, options: QueryOptions, statement: string) {
  helper( // TWO: both original reference identities must survive.
    query,
    options,
  );
  read( // CLEAR: the known SQL call must not swallow the nested unknown.
    'SELECT 1',
    helper( // ONE: nearest nested invocation owns the uncertainty.
      query,
    ),
  );
  read('SELECT 1', options); // CLEAR: proven metadata stays exempt.
  helper(
    () => (
      query // ONE: deferred body has no enclosing runtime invocation owner.
    ),
  );
  let alias: any;
  helper( // ONE: the opaque invocation owns the transferred executor uncertainty.
    alias = query, // CLEAR: the transfer itself does not execute SQL.
  );
  helper(
    class { field = query }, // ONE: class body creates an independent escape.
  );
  query(statement); // ONE: actual SQL calls retain their normal location.
  read('SELECT 1', { onRetry: write }); // ONE: executable callback lacks a contract.
  read('SELECT 1', [write]); // ONE: executable array value lacks a contract.
  read(SQL`SELECT ${{ onRetry: write }} AS value`); // CLEAR: library interpolation is a bind.
  read(SQL`SELECT ${[write]} AS value`); // CLEAR: a bound array remains data.
  helper( // ONE: erased cast runtime producer anchors to the enclosing call.
    value as QueryExecutor,
  );
}
export class RuntimeReceiver {
  options!: QueryOptions;
  escape() {
    helper( // ONE: class field runtime span uses the same location map.
      this.options,
    );
  }
}
export function suppressedEscape(query: QueryExecutor) {
  // no-mistakes-disable-next-line postgres-no-offset
  helper( // CLEAR: suppression belongs to the displayed invocation line.
    query,
  );
}

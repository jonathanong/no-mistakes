import { read, write } from '@example/db';
import * as db from '@example/db';

// Every shape here mentions an executor without invoking it; none may report.
export function references(options: any = db) {
  if (options.query || options.client) return withTransactionOptions(options, run);
  if (!options.query) {
    return null;
  }
  const run = options.query ?? write;
  const runQuery = options.query ? write : read;
  const { rows } = options.query;
  const typed = options.query as TransactionQuery;
  register(options.query, () => 0);
  consume({ query: options.query, vote: false });
  consume(options.query === db.query, db.query, new Proxy(db.write, {}));
  return [run, runQuery, rows, typed];
}

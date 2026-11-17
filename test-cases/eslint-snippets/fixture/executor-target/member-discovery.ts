import { read } from '@example/other';
import type { QueryExecutor } from '@example/db';

const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
helpers.load(read);
function shadowed(read: (statement: string) => unknown) {
  helpers.load(read);
}

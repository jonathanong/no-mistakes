import { read } from '@example/db';
import type { QueryOptions } from '@example/executor-types';
declare const dynamicSql: string;
declare const options: QueryOptions;

const nullishQuery = options.query ?? read; // NON_SQL
nullishQuery(dynamicSql); // ADVISE_NULLISH_QUERY
const fallbackQuery = options.query || read; // NON_SQL
fallbackQuery(dynamicSql); // ADVISE_OR_QUERY
if (options.query) {
  const guardedQuery = options.query; // NON_SQL
  guardedQuery(dynamicSql); // ADVISE_GUARDED_QUERY
  read('SELECT 1'); // CLEAR_GUARDED_READ
}

import { read } from '@example/db';
import type { QueryOptions } from '@example/db';
import SQL from 'sql-template-strings';
export function mutated(options: QueryOptions, replacement: any) {
  read(SQL`SELECT ${options.id} AS id`); // BIND_UNKNOWN: later write invalidates this exact tag.
  SQL = replacement;
}

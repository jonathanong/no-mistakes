import type { QueryExecutor } from '@example/db';
import { build } from './failure-helper.mjs';

export function selected(query: QueryExecutor, value: string) {
  query(build(value));
}

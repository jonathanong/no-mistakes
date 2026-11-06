import type { QueryExecutor } from '@example/db';
import { forward } from './failed-helper.mjs';
export function run(first: QueryExecutor, second: QueryExecutor) {
  forward(first, second);
}

import { type QueryExecutor } from '@example/db';
import { type QueryExecutor as OtherExecutor } from '@unrelated/db';
import * as namespace from '@example/db';

// Same spellings, unions, namespaces and nested object types are not direct
// references to the configured type import, so they cannot become SQL proofs.
export function shadow<QueryExecutor>(executor: QueryExecutor) {
  executor('SELECT id FROM orders OFFSET 1');
}
export function unrelated(executor: OtherExecutor) {
  executor('SELECT id FROM orders OFFSET 1');
}
export function union(executor: QueryExecutor | undefined) {
  executor('SELECT id FROM orders OFFSET 1');
}
export function qualified(executor: namespace.QueryExecutor) {
  executor('SELECT id FROM orders OFFSET 1');
}
export function plain(executor) {
  executor('SELECT id FROM orders OFFSET 1');
}
export function shapes(
  { nested: { executor } }: { nested: { executor: QueryExecutor } },
  [array]: QueryExecutor[],
  { plain }: { plain: string; method(): void; missing; other: number },
  { value }: { unmatched: QueryExecutor },
) {
  executor('SELECT id FROM orders OFFSET 1');
}

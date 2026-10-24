import SQL from 'sql-template-strings';
import { read, type QueryExecutor } from '@example/db';
import { asyncFragment, mixedFragment, offsetFragment } from './helpers.mjs';

export async function policyConsumers(query: QueryExecutor, enabled: boolean, id: string) {
  read(SQL`SELECT 1`.append(offsetFragment({ offset: true }))); // HELPER_OFFSET
  read(SQL`SELECT 1`.append(mixedFragment(enabled))); // OFFSET_WITH_RESIDUAL
  read(SQL`SELECT 1 FROM ${process.env.UNSAFE_TABLE}`); // UNBOUNDED_IDENTIFIER
  read(SQL`SELECT 1 FROM accounts WHERE id::text = ${id}`); // TRANSFORMED_UUID
  read(SQL`SELECT 1 FROM accounts WHERE id::text = ${id} AND `.append(mixedFragment(enabled))); // UUID_WITH_RESIDUAL
  read(SQL`SELECT 1`.append(await asyncFragment())); // ASYNC_FRAGMENT
  read(SQL`SELECT 1`.append(asyncFragment())); // UNAWAITED_PROMISE
  await query(SQL`SELECT 1`);
}

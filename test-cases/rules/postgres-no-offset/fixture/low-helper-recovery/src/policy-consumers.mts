import SQL from 'sql-template-strings';
import { read, type QueryExecutor } from '@example/db';
import { asyncFragment, mixedFragment, offsetFragment } from './helpers.mjs';

export async function policyConsumers(executor: QueryExecutor, enabled: boolean, id: string) {
  read(SQL`SELECT 1`.append(offsetFragment({ offset: true }))); // HELPER_OFFSET
  read(SQL`SELECT 1`.append(mixedFragment(enabled))); // OFFSET_WITH_RESIDUAL
  executor(`SELECT 1 FROM ${process.env.UNSAFE_TABLE}.items`); // UNBOUNDED_IDENTIFIER
  read('SELECT 1 FROM accounts WHERE id::text = $1'); // TRANSFORMED_UUID
  read(SQL`SELECT 1 FROM accounts WHERE id::text = $1`.append(mixedFragment(enabled))); // UUID_WITH_RESIDUAL
  read(SQL`SELECT 1`.append(await asyncFragment())); // ASYNC_FRAGMENT
  read(SQL`SELECT 1`.append(asyncFragment())); // UNAWAITED_PROMISE
  await executor(SQL`SELECT 1`);
}

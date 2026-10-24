import SQL from 'sql-template-strings';
import { read, type QueryExecutor } from '@example/db';
import { asyncForward, asyncFragment, asyncCarrier, unknownAwait, builderCarrier, nestedBuilderCarrier, consumeBuilderBind } from './helpers.mjs';
export async function execute(query: QueryExecutor, unknown: unknown) {
  await asyncForward(query, 'SELECT 1'); // CLEAR_ASYNC
  builderCarrier(query); // UNKNOWN_RETURNED_BUILDER
  nestedBuilderCarrier(query); // UNKNOWN_NESTED_BUILDER
  consumeBuilderBind(query); // CLEAR_CONSUMED_BIND
  asyncCarrier(query); // UNKNOWN_CARRIER
  await unknownAwait(query, unknown); // UNKNOWN_THENABLE
  read(SQL`SELECT 1`.append(await asyncFragment())); // CLEAR_AWAITED_BUILDER
  read(SQL`SELECT 1`.append(asyncFragment())); // UNKNOWN_PROMISE_SQL
}

import { beginTransaction, beginBoundedTransaction, executeHandlerWithCursorInBatches as batches, createAsyncGeneratorFromCursor as cursor, read } from './psql.mjs';
import { session } from '@vendor/valkey';
import { readLatch, diagnostics, wrapper } from './helpers.mjs';
export async function entry(items: string[]) {
  await using transaction = await beginTransaction();
  const recovery = await beginBoundedTransaction();
  for (const item of items) {
    await transaction('SELECT 1');
    await recovery('SELECT 1');
    await batches('SELECT 1');
    await cursor('SELECT 1');
    await readLatch();
    diagnostics(session);
    wrapper(session);
    read('SELECT 1');
    rejected(item);
    prepared.finalize();
  }
}
// These collisions must stay quiet despite the configured executor spellings.
function rejected(item: string) { return item; }
const prepared = { finalize() {} };
import { begin as opaqueBegin } from '@vendor/opaque-database';
export async function opaque(items: string[]) {
  const txQuery = await opaqueBegin();
  for (const item of items) { await txQuery(item); }
}

/* oxlint-disable no-await-in-loop */
import type { TransactionQuery } from '@pkg/psql';
import { CHILD_STAGES } from './stages.mts';
async function checkpoint(query: TransactionQuery, id: string) {}
export async function purge(query: TransactionQuery, id: string, n: number) {
  for (const stage of CHILD_STAGES) {
    const deleted = await stage.deletion(query, id, n);
    if ((deleted.rowCount ?? 0) > 0) {
      await checkpoint(query, id);
      return { done: false };
    }
    const remains = await stage.remaining(query, id);
    if (remains.rows[0]?.remaining) {
      await checkpoint(query, id);
      return { done: false };
    }
  }
  return { done: true };
}

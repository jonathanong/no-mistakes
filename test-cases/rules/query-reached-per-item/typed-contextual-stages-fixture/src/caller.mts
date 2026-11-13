/* oxlint-disable no-await-in-loop */
import type { TransactionQuery } from '@pkg/psql';
import { CHILD_STAGES } from './stages.mts';
async function checkpoint(query: TransactionQuery, id: string) {}
export async function purge(query: TransactionQuery, candidates: { id: string }[], batchSize: number) {
  for (const candidate of candidates) {
    for (const stage of CHILD_STAGES) {
      const deleted = await stage.deletion(query, candidate.id, batchSize);
      if ((deleted.rowCount ?? 0) > 0) {
        await checkpoint(query, candidate.id);
        return { done: false };
      }
      const remains = await stage.remaining(query, candidate.id);
      if (remains.rows[0]?.remaining) {
        await checkpoint(query, candidate.id);
        return { done: false };
      }
    }
  }
  return { done: true };
}

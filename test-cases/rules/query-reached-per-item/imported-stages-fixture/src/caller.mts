import type { TransactionQuery } from './database.mts';
import { STAGES } from './stages.mts';
export async function cleanup(query: TransactionQuery, id: string, n: number) {
  for (const stage of STAGES) {
    const r = await stage.run(query, id, n);
    const c = await stage.check(query, id);
  }
}

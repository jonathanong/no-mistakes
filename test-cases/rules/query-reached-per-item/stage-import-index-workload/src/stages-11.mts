import type { TransactionQuery } from './database.mts';
type Stage = {
  run: (query: TransactionQuery, id: string, n: number) => Promise<{ rowCount: number | null }>;
  check: (query: TransactionQuery, id: string) => Promise<{ rows: { remaining: boolean }[] }>;
};
export const STAGES_11: readonly Stage[] = [
  {
    run: (query, id, n) => query('DELETE FROM cleanup_11 WHERE parent_id = $1 LIMIT $2', [id, n]),
    check: (query, id) => query('SELECT EXISTS (SELECT 1 FROM cleanup_11 WHERE parent_id = $1) AS remaining', [id]),
  },
];

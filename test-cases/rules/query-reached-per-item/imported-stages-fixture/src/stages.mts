import type { TransactionQuery } from './database.mts';
export type Stage = {
  run: (query: TransactionQuery, id: string, n: number) => Promise<{ rowCount: number | null }>;
  check: (query: TransactionQuery, id: string) => Promise<{ rows: { remaining: boolean }[] }>;
};
export const STAGES: readonly Stage[] = [
  {
    run: (query, id, n) => query('DELETE FROM a WHERE parent_id = $1 LIMIT $2', [id, n]),
    check: (query, id) => query('SELECT EXISTS (SELECT 1 FROM a WHERE parent_id = $1) AS remaining', [id]),
  },
  {
    run: (query, id, n) => query('DELETE FROM b WHERE parent_id = $1 LIMIT $2', [id, n]),
    check: (query, id) => query('SELECT EXISTS (SELECT 1 FROM b WHERE parent_id = $1) AS remaining', [id]),
  },
];

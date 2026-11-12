import type { TransactionQuery } from '@pkg/psql';
export type ChildStage = {
  deletion: (query: TransactionQuery, id: string, n: number) => Promise<{ rowCount: number | null }>;
  remaining: (query: TransactionQuery, id: string) => Promise<{ rows: { remaining: boolean }[] }>;
};
export const CHILD_STAGES: readonly ChildStage[] = [
  {
    deletion: (query, id, n) => query(`/* deleteAPage */ WITH page AS (SELECT id FROM a WHERE parent_id = $1 LIMIT $2) DELETE FROM a USING page WHERE a.id = page.id`, [id, n]),
    remaining: (query, id) => query<{ remaining: boolean }>(`/* hasA */ SELECT EXISTS (SELECT 1 FROM a WHERE parent_id = $1) AS remaining`, [id]),
  },
  {
    deletion: (query, id, n) => query(`/* deleteBPage */ WITH page AS (SELECT id FROM b WHERE parent_id = $1 LIMIT $2) DELETE FROM b USING page WHERE b.id = page.id`, [id, n]),
    remaining: (query, id) => query<{ remaining: boolean }>(`/* hasB */ SELECT EXISTS (SELECT 1 FROM b WHERE parent_id = $1) AS remaining`, [id]),
  },
];

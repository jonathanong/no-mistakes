import type { TransactionQuery } from '@pkg/psql';
type PlainQuery = (sql: string, values: unknown[]) => Promise<void>;
type PlainStage = {
  run: (query: PlainQuery, id: string) => Promise<void>;
};
const PLAIN_STAGES: readonly PlainStage[] = [
  { run: (query, id) => query('plain stage', [id]) },
];
export async function plain(query: PlainQuery, id: string) {
  for (const stage of PLAIN_STAGES) {
    await stage.run(query, id);
  }
}
export type { TransactionQuery };

import { type Query } from 'db';
import { createTempTableName } from './temp-names.ts';

async function prepareOpen(q: Query, prefix: string): Promise<string> {
  const table = createTempTableName(prefix);
  await q(`/* prepareOpen */ CREATE TEMP TABLE ${table} (id uuid) ON COMMIT DROP`);
  return table;
}

export async function pruneOpen(q: Query, prefix: string) {
  const table = await prepareOpen(q, prefix);
  await q(`/* pruneOpen */ DELETE FROM items i USING ${table} r WHERE r.id = i.id`);
}

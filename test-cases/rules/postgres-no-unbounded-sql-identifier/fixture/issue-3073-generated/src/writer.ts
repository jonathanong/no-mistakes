import { type Query } from 'db';
import { createTempTableName } from './temp-names.ts';

async function prepare(q: Query, ids: string[]): Promise<string> {
  const table = createTempTableName('temp_recipients');
  await q(`/* prepare */ CREATE TEMP TABLE ${table} (id uuid) ON COMMIT DROP`);
  return table;
}

export async function prune(q: Query, ids: string[]) {
  const table = await prepare(q, ids);
  await q(`/* prune */ DELETE FROM items i USING ${table} r WHERE r.id = i.id`);
}

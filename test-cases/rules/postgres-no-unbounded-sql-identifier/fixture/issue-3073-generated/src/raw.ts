import { type Query } from 'db';

export async function pruneRaw(q: Query, table: string) {
  await q(`/* pruneRaw */ DELETE FROM items i USING ${table} r WHERE r.id = i.id`);
}

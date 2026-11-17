import { beginTransaction } from 'db';
import { getConfig } from './config.mts';

export async function insertImpact(name: string, id: string) {
  const config = getConfig(name);
  const query = await beginTransaction();
  await query(
    `/* insertImpact */ INSERT INTO impacts (request_id, ${config.column}) VALUES ($1, $2)`,
    [id, id],
  );
  await query(`/* insertTable */ INSERT INTO ${config.table} (id) VALUES ($1)`, [id]);
}

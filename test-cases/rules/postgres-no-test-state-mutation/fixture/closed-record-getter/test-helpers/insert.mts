import { beginTransaction } from 'db';
import { getConfig, getEarly } from './config.mts';

export async function insertImpact(id: string) {
  const config = getConfig('a');
  const early = getEarly('b');
  const query = await beginTransaction();
  await query(
    `/* c */ INSERT INTO impacts (request_id, subject_id, ${config.column}) VALUES ($1, $2, $3)`,
    [id, id, id],
  );
  await query(
    `/* early */ INSERT INTO impacts (request_id, subject_id, ${early.column}) VALUES ($1, $2, $3)`,
    [id, id, id],
  );
}

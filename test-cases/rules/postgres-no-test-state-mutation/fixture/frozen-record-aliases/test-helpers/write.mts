import { beginTransaction } from 'db';
import sql from 'sql-template-strings';
import { getConfig } from './config.mts';

export async function writeConst(name: string) {
  const config = getConfig(name);
  const query = await beginTransaction();
  await query(sql`INSERT INTO impacts (id, ${config.column}) VALUES ($1, $2)`);
}

export async function writeLet(name: string) {
  let config = getConfig(name);
  const query = await beginTransaction();
  await query(sql`INSERT INTO impacts (id, ${config.column}) VALUES ($1, $2)`);
}

export async function writeReassigned(name: string, column: string) {
  let config: { column: string } = getConfig(name);
  config = { column };
  const query = await beginTransaction();
  await query(sql`INSERT INTO impacts (id, ${config.column}) VALUES ($1, $2)`);
}

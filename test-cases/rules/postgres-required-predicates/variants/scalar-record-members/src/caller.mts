import { query } from '@example/db';
import sql from 'sql-template-strings';
import { config, scalar } from './records.mts';
export async function mixed(status: string, at: Date, id: string) {
  const fields = scalar(status, at);
  const selected = config('one');
  await query(sql`/* mixed */ SELECT ${selected.column}, ${fields.aAt}, ${id}, ${fields.aAt} FROM ${selected.table} WHERE ${selected.column} = ${id}`);
}
export async function ordinary(status: string, at: Date) {
  const fields = scalar(status, at);
  await query(`/* ordinary */ SELECT ${fields.aAt}`);
}
export async function relation(status: string, at: Date) {
  const fields = scalar(status, at);
  await query(sql`/* relation */ SELECT id FROM ${fields.aAt}`);
}

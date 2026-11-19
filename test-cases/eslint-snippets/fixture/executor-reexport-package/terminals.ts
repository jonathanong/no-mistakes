import sql, { type Statement } from "sql-template-strings";
import { read, write, type QueryExecutor, type QueryOptions as Options } from "@data-stores/psql";

interface Widget {
  id: string;
  state: string;
}

async function queryById(id: string, run: (statement: Statement) => Promise<unknown>) {
  return run(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function getFromPrimary(id: string, options?: Options) {
  return queryById(id, (statement) => write(statement, { ...options, readOnly: false }));
}

async function assertDelivered(id: string, options: Options = {}) {
  await getFromPrimary(id, options);
}

async function readBare(id: string, options: Options = {}) {
  return read(sql`SELECT 1 FROM widgets WHERE id = ${id}`, options);
}

async function writeDirectSpread(id: string, options: Options = {}) {
  return write(sql`UPDATE widgets SET s = 1 WHERE id = ${id}`, { ...options, readOnly: false });
}

export async function t1(query: QueryExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, state FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return assertDelivered(id, { query });
  return assertDelivered(row.id, { query });
}

export async function t2(query: QueryExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, state FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return assertDelivered(id, { query });
  return readBare(row.id, { query });
}

export async function t3(query: QueryExecutor, id: string) {
  const { rows } = await query<Widget>(sql`SELECT id, state FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return assertDelivered(id, { query });
  return writeDirectSpread(row.id, { query });
}

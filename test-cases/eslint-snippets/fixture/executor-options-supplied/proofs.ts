import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };
type Result = { rows: Widget[] };
interface QueryOptions {
  label?: string;
  query?: SessionExecutor;
  readOnly?: boolean;
}

async function assertDelivered(id: string, options: QueryOptions = {}) {
  await options.query?.(sql`SELECT ${id}`);
}

async function rejectUnresolvable(id: string, options: QueryOptions = {}): Promise<never> {
  await assertDelivered(id, options);
  throw new Error("not found");
}

async function rejectDeeper(id: string, options: QueryOptions = {}): Promise<never> {
  await rejectUnresolvable(id, options);
  throw new Error("not found");
}

async function rejectOptional(id: string, options?: { query: SessionExecutor }) {
  await options!.query(sql`SELECT ${id}`);
}

export async function oneLevel(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  if (!rows[0]) return rejectUnresolvable(id, { query });
  return rows[0].state;
}

export async function twoLevels(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  if (!rows[0]) return rejectDeeper(id, { query });
  return rows[0].state;
}

export async function renamedProperty(tx: SessionExecutor, id: string) {
  const { rows } = (await tx(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  if (!rows[0]) return rejectUnresolvable(id, { query: tx });
  return rows[0].state;
}

export async function optionalParameter(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  if (!rows[0]) return rejectOptional(id, { query });
  return rows[0].state;
}

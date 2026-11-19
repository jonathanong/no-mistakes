import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as appendResolvers from "./appendResolvers";

type Widget = { id: string; state: string };
type Result = { rows: Widget[] };
interface RunOptions {
  client?: object;
  query?: SessionExecutor;
  readOnly?: boolean;
}

async function leafDefault(id: string, options: RunOptions = {}) {
  await options.query!(sql`UPDATE widgets SET state = 'r' WHERE id = ${id}`);
}

async function leafDynamic(id: string, column: string, options: RunOptions = {}) {
  await options.query!(sql`UPDATE widgets SET state = 'r' WHERE id = ${id}`.append(column));
}

async function hopDynamicCol(id: string, column: string, options: RunOptions = {}) {
  await leafDynamic(id, column, options);
}

async function hopMutates(id: string, options: RunOptions = {}) {
  options.readOnly = true;
  await leafDefault(id, options);
}

export async function r9(query: SessionExecutor, id: string, column: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) return null;
  return leafDynamic(row.id, column, { query });
}

export async function r10(query: SessionExecutor, id: string, column: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) return null;
  return hopDynamicCol(row.id, column, { query });
}

export async function r11(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  if (!rows[0]) return hopMutates(id, { query });
  return rows[0].state;
}

export async function r13(query: SessionExecutor, id: string, column: string) {
  return appendResolvers.appendDynamic(id, column, { query });
}

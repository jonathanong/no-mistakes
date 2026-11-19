import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };
type Result = { rows: Widget[] };

function rejectUntyped(id: string, options) {
  return options.query(sql`SELECT ${id}`);
}

function rejectAny(id: string, options: any) {
  return options.query(sql`SELECT ${id}`);
}

function rejectUnknown(id: string, options: { query: unknown }) {
  return (options.query as SessionExecutor)(sql`SELECT ${id}`);
}

function rejectFunction(id: string, options: { query: (statement: unknown) => unknown }) {
  return options.query(sql`SELECT ${id}`);
}

function rejectOptionalProperty(id: string, options: { query?: SessionExecutor }) {
  return options.query?.(sql`SELECT ${id}`);
}

function rejectInferred(id: string, options = { query: undefined as unknown as SessionExecutor }) {
  return options.query(sql`SELECT ${id}`);
}

const stash: unknown[] = [];
function rejectEscaping(id: string, options?: { query: SessionExecutor }) {
  stash.push(options);
  return options?.query(sql`SELECT ${id}`);
}

async function read(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  return rows[0];
}

export async function untyped(query: SessionExecutor, id: string) {
  const row = await read(query, id);
  if (!row) return rejectUntyped(id, { query });
  return row.state;
}

export async function anyOptions(query: SessionExecutor, id: string) {
  const row = await read(query, id);
  if (!row) return rejectAny(id, { query });
  return row.state;
}

export async function unknownProperty(query: SessionExecutor, id: string) {
  const row = await read(query, id);
  if (!row) return rejectUnknown(id, { query });
  return row.state;
}

export async function functionProperty(query: SessionExecutor, id: string) {
  const row = await read(query, id);
  if (!row) return rejectFunction(id, { query });
  return row.state;
}

export async function optionalProperty(query: SessionExecutor, id: string) {
  const row = await read(query, id);
  if (!row) return rejectOptionalProperty(id, { query });
  return row.state;
}

export async function inferredOptions(query: SessionExecutor, id: string) {
  const row = await read(query, id);
  if (!row) return rejectInferred(id, { query });
  return row.state;
}

export async function escapingOptions(query: SessionExecutor, id: string) {
  const row = await read(query, id);
  if (!row) return rejectEscaping(id, { query });
  return row.state;
}

async function markWidget(id: string, query: SessionExecutor) {
  await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function andGuard(query: SessionExecutor, id: string, locked: boolean) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row && locked) return null;
  return markWidget(row.id, query);
}

export async function orGuardNoExit(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row || row.state === "locked") stash.push(id);
  return markWidget(row.id, query);
}

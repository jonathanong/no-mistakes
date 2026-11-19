import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };
type Result = { rows: Widget[] };
type RejectOptions = { query: SessionExecutor };

function rejectDeclared(id: string, options: RejectOptions) {
  return options.query(sql`SELECT ${id}`);
}

function rejectOptionalParameter(id: string, options?: RejectOptions) {
  return options?.query(sql`SELECT ${id}`);
}

function rejectDefaulted(id: string, { query }: RejectOptions = invalidDefault()) {
  return query(sql`SELECT ${id}`);
}

declare function invalidDefault(): never;

export async function declaredOptions(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) return rejectDeclared(id, { query });
  return row.state;
}

export async function optionalParameter(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) return rejectOptionalParameter(id, { query });
  return row.state;
}

export async function defaultedParameter(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) return rejectDefaulted(id, { query });
  return row.state;
}

function rejectNullable(id: string, options: RejectOptions | null) {
  return options?.query(sql`SELECT ${id}`);
}

export async function nullableOptions(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) return rejectNullable(id, { query });
  return row.state;
}

async function markWidget(id: string, query: SessionExecutor) {
  await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function orGuard(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row || row.state === "locked") return null;
  return markWidget(row.id, query);
}

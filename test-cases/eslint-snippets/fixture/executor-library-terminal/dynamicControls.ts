import { write, type SessionExecutor } from "@example/db";
import sql, { type SqlStatement } from "@example/sql";
import { rawSql } from "./untrusted";

interface QueryOptions {
  label?: string;
  query?: SessionExecutor;
  readOnly?: boolean;
}
interface Widget {
  id: string;
  state: string;
}

async function byIdAppended(
  id: string,
  column: string,
  run: (s: SqlStatement) => Promise<unknown>,
) {
  return run(sql`SELECT id FROM widgets WHERE id = ${id}`.append(column));
}

async function byIdSuffixed(
  id: string,
  suffix: string,
  run: (s: SqlStatement) => Promise<unknown>,
) {
  return run(sql`SELECT id FROM widgets WHERE id = ${id}`.append(suffix));
}

async function byIdUntrusted(id: string, run: (s: SqlStatement) => Promise<unknown>) {
  return run(rawSql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function byIdMixed(id: string, column: string, run: (s: SqlStatement) => Promise<unknown>) {
  if (id === "") return run(sql`SELECT id FROM widgets`);
  if (id === "x") return run(sql`SELECT id FROM widgets WHERE id = ${id}`);
  return run(sql`SELECT id FROM widgets WHERE id = ${id}`.append(column));
}

async function appendedAdapter(id: string, column: string, options?: QueryOptions) {
  return byIdAppended(id, column, (statement) => write(statement, { ...options }));
}

async function suffixedAdapter(id: string, suffix: string, options: QueryOptions = {}) {
  return byIdSuffixed(id, suffix, (statement) => write(statement, options));
}

async function untrustedAdapter(id: string, options: QueryOptions = {}) {
  return byIdUntrusted(id, (statement) => write(statement, options));
}

async function mixedAdapter(id: string, column: string, options: QueryOptions = {}) {
  return byIdMixed(id, column, (statement) => write(statement, options));
}

export async function t4(query: SessionExecutor, id: string, column: string) {
  const { rows } = (await query(sql`SELECT id, state FROM widgets WHERE id = ${id}`)) as {
    rows: Widget[];
  };
  const row = rows[0];
  if (!row) return null;
  return appendedAdapter(row.id, column, { query });
}

export async function variants(query: SessionExecutor, id: string, column: string) {
  await suffixedAdapter(id, column, { query });
  await untrustedAdapter(id, { query });
  await mixedAdapter(id, column, { query });
}

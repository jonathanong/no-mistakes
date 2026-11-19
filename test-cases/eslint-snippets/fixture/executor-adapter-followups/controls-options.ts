import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; status: string };
type RunOptions = { query?: SessionExecutor; readOnly?: boolean };

declare function stash(value: object): void;

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function spreadOptions(id: string, options: RunOptions = {}) {
  const copy = { ...options };
  await copy.query?.(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function restOptions(id: string, opts: RunOptions = {}) {
  const { query, ...rest } = opts;
  stash(rest);
  await query?.(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function parenWrite(id: string, options: RunOptions = {}) {
  (options.readOnly) = true;
  await options.query?.(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function destructureWrite(id: string, options: RunOptions = {}, src: RunOptions) {
  ({ readOnly: options.readOnly } = src);
  await options.query?.(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function parenPattern(id: string, options: RunOptions = {}, src: RunOptions) {
  ({ readOnly: (options.readOnly) } = src);
  await options.query?.(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function selfOptions(id: string, options: RunOptions = {}): Promise<unknown> {
  return id ? selfOptions(id, options) : options;
}

export async function spreadCaller(query: SessionExecutor, id: string) {
  await spreadOptions(id, { query });
}

export async function restCaller(query: SessionExecutor, id: string) {
  await restOptions(id, { query });
}

export async function parenCaller(query: SessionExecutor, id: string) {
  await parenWrite(id, { query });
}

export async function destructureCaller(query: SessionExecutor, id: string) {
  await destructureWrite(id, { query }, { query });
}

export async function parenPatternCaller(query: SessionExecutor, id: string) {
  await parenPattern(id, { query }, { query });
}

export async function coalesce(query: SessionExecutor, id: string) {
  const { rows } = await query<{ rows: Notice[] }>(sql`SELECT id FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (row ?? null) return lockNotice(query, row.id);
  return null;
}

export async function recurse(query: SessionExecutor, id: string) {
  await selfOptions(id, { query });
}

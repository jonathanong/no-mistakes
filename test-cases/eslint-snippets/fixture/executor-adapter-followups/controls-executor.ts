import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; status: string };
type LocalExecutor = (statement: string) => Promise<{ rows: Notice[] }>;

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

export async function reassigned(query: SessionExecutor, other: SessionExecutor, id: string) {
  const { rows } = await query<{ rows: Notice[] }>(sql`SELECT id FROM notices WHERE id = ${id}`);
  query = other;
  return lockNotice(query, rows[0]!.id);
}

export async function shadowed(query: SessionExecutor, id: string) {
  const lookup = async () => ({ rows: [] as Notice[] });
  const alias = lookup;
  const { rows } = await alias();
  return lockNotice(query, rows[0]!.id);
}

export async function notExecutorTyped(query: SessionExecutor, other: LocalExecutor, id: string) {
  const { rows } = await other(sql`SELECT id FROM notices WHERE id = ${id}`);
  return lockNotice(query, rows[0]!.id);
}

export async function closureWrite(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  const write = () => {
    notice.id = "x";
  };
  write();
  return lockNotice(query, notice.id);
}

export async function parameterAlias(query: SessionExecutor, id: string) {
  const next = query;
  const { rows } = await next<{ rows: Notice[] }>(sql`SELECT id FROM notices WHERE id = ${id}`);
  return lockNotice(query, rows[0]!.id);
}

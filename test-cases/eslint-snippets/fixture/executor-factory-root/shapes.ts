import defaultOpener, { beginTransaction, type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };

declare const pending: Promise<SessionExecutor>;
declare const holder: {
  begin(): Promise<SessionExecutor>;
  run<T>(statement: unknown): Promise<{ rows: T[] }>;
};

async function markWidget(id: string, executor: SessionExecutor) {
  await executor(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function notAwaited(query: SessionExecutor) {
  const tx = beginTransaction() as unknown as SessionExecutor;
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function notCall(query: SessionExecutor) {
  const tx = await pending;
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function memberCallee(query: SessionExecutor) {
  const tx = await holder.begin();
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function defaultImport(query: SessionExecutor) {
  const tx = await defaultOpener();
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function destructured(query: SessionExecutor) {
  const { tx } = { tx: await beginTransaction() };
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function unresolved(query: SessionExecutor) {
  const { rows } = await missing<Widget>(sql`SELECT id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function memberReceiver(query: SessionExecutor) {
  const { rows } = await holder.run<Widget>(sql`SELECT id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function notAwaitedResult(query: SessionExecutor) {
  const { rows } = query<Widget>(sql`SELECT id FROM widgets`) as unknown as { rows: Widget[] };
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function forOfBinding(query: SessionExecutor, sessions: SessionExecutor[]) {
  for (const tx of sessions) {
    const { rows } = await tx<Widget>(sql`SELECT id FROM widgets`);
    const row = rows[0];
    if (!row) return null;
    return markWidget(row.id, query);
  }
}

export async function untypedParameter(
  query: SessionExecutor,
  run: (statement: unknown) => Promise<{ rows: Widget[] }>,
  qualified: import("@example/db").SessionExecutor,
) {
  const { rows } = await run(sql`SELECT id FROM widgets`);
  const { rows: more } = await qualified<Widget>(sql`SELECT id FROM widgets`);
  const row = rows[0] ?? more[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function redeclared(query: SessionExecutor) {
  var tx = await beginTransaction();
  var tx = await beginTransaction();
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

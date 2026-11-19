import { type SessionExecutor, write } from "@example/db";
import sql from "@example/sql";

type RunOptions = { query?: SessionExecutor };

export async function bangReceiver(id: string, options?: RunOptions) {
  await options!.query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function selectWidget(id: string, execute: SessionExecutor) {
  return execute(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

export async function fromClosure(id: string, options?: RunOptions) {
  return selectWidget(id, (statement) => write(statement, options));
}

function takeUnknown(id: string, value: unknown) {
  return { id, value };
}

export async function badClosure(id: string, options?: RunOptions) {
  return takeUnknown(id, (statement) => write(statement, options));
}

export function writeThree(statement: unknown, options: unknown, extra: unknown): Promise<unknown> {
  return write(statement as never, options as never);
}

export async function extraArgument(id: string, options: RunOptions = {}) {
  return selectWidget(id, (statement) => writeThree(statement, options, id));
}

export async function eachId(ids: string[], tx: SessionExecutor) {
  for (const id of ids) {
    await tx(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
  }
}

type NoticeRow = { id: string };

export async function eachField(rows: NoticeRow[], tx: SessionExecutor) {
  for (const row of rows) {
    await tx(sql`UPDATE widgets SET state = 'x' WHERE id = ${row.id}`);
  }
}

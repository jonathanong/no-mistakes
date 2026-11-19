import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as appendResolvers from "./appendResolvers";

interface RunOptions {
  query?: SessionExecutor;
}

async function leafAppend(id: string, options: RunOptions = {}) {
  await options.query!(
    sql`UPDATE widgets SET state = 'r' WHERE id = ${id}`.append(sql` AND state = 'new'`),
  );
}

async function hopAppend(id: string, options: RunOptions = {}) {
  await leafAppend(id, options);
}

export async function localLiteral(query: SessionExecutor, id: string) {
  return leafAppend(id, { query });
}

export async function forwardedLiteral(query: SessionExecutor, id: string) {
  return hopAppend(id, { query });
}

export async function importedLiteral(query: SessionExecutor, id: string) {
  return appendResolvers.appendLiteral(id, { query });
}

import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export interface RunOptions {
  query?: SessionExecutor;
}

export async function appendLiteral(id: string, options: RunOptions = {}) {
  await options.query!(sql`UPDATE widgets SET state = 'r' WHERE id = ${id}`.append("-- tag"));
}

export async function appendDynamic(id: string, column: string, options: RunOptions = {}) {
  await options.query!(sql`UPDATE widgets SET state = 'r' WHERE id = ${id}`.append(column));
}

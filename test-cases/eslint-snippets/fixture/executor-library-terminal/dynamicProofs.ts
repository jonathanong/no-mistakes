import { write, type SessionExecutor } from "@example/db";
import sql, { type SqlStatement } from "@example/sql";

interface QueryOptions {
  query?: SessionExecutor;
  readOnly?: boolean;
}

async function byIdProven(id: string, run: (s: SqlStatement) => Promise<unknown>) {
  return run(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function provenAdapter(id: string, options?: QueryOptions) {
  return byIdProven(id, (statement) => write(statement, { ...options, readOnly: false }));
}

export async function provenChain(query: SessionExecutor, id: string) {
  await provenAdapter(id, { query });
}

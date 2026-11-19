import sql, { type Statement } from "./sql.mjs";
import { write, type QueryOptions, type QueryExecutor } from "./db.mjs";

interface Widget {
  id: string;
  state: string;
}

async function queryByIdDynamic(
  id: string,
  column: string,
  run: (statement: Statement) => Promise<unknown>,
) {
  return run(sql`SELECT id FROM widgets WHERE id = ${id}`.append(column));
}

async function dynamicAdapter(id: string, column: string, options?: QueryOptions) {
  return queryByIdDynamic(id, column, (statement) => write(statement, { ...options }));
}

export async function t4(query: QueryExecutor, id: string, column: string) {
  const { rows } = await query<Widget>(sql`SELECT id, state FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return null;
  return dynamicAdapter(row.id, column, { query });
}

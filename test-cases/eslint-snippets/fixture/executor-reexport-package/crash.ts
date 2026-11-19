import sql from "sql-template-strings";
import { read, type QueryExecutor as Executor, type RecOptions } from "@data-stores/psql";

async function b(options: RecOptions = { x: "1" }) {
  return read(sql`SELECT 1 FROM widgets`, options);
}

export async function reports(query: Executor) {
  await b({ query, x: "1" });
}

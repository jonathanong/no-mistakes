import sql from "sql-template-strings";
import { read, type QueryExecutor as Executor } from "@data-stores/psql";
import type { QueryOptions as OtherOptions } from "@other/pg-core";
import type { QueryOptions as BrokenOptions } from "@broken/pg-core";

async function viaOther(id: string, options: OtherOptions = {}) {
  return read(sql`SELECT 1 FROM widgets WHERE id = ${id}`, options);
}

async function viaBroken(id: string, options: BrokenOptions = {}) {
  return read(sql`SELECT 1 FROM widgets WHERE id = ${id}`, options);
}

export async function reports(query: Executor, id: string) {
  await viaOther(id, { query });
  await viaBroken(id, { query });
}

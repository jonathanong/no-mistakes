import sql from "sql-template-strings";
import {
  read,
  type LooseOptions,
  type QueryExecutor as Executor,
  type UnresolvedOptions,
  type WideOptions,
} from "@data-stores/psql";
import type { QueryOptions as ForeignOptions } from "@other/pg-core";

type QueryExecutor = (statement: { text: string }) => Promise<unknown>;
interface QueryOptions {
  query?: QueryExecutor;
}

async function viaForeign(id: string, options: ForeignOptions = {}) {
  return read(sql`SELECT 1 FROM widgets WHERE id = ${id}`, options);
}

async function viaLocal(id: string, options: QueryOptions = {}) {
  return read(sql`SELECT 1 FROM widgets WHERE id = ${id}`, options);
}

async function viaLoose(id: string, options: LooseOptions = {}) {
  return read(sql`SELECT 1 FROM widgets WHERE id = ${id}`, options);
}

async function viaUnresolved(id: string, options: UnresolvedOptions = {}) {
  return read(sql`SELECT 1 FROM widgets WHERE id = ${id}`, options);
}

async function viaWide(id: string, options: WideOptions = {}) {
  return read(sql`SELECT 1 FROM widgets WHERE id = ${id}`, options);
}

export async function reports(query: Executor, id: string) {
  await viaForeign(id, { query });
  await viaLocal(id, { query });
  await viaLoose(id, { query });
  await viaWide(id, { query });
  await viaUnresolved(id, { query });
}

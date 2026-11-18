import { read, type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { requireValue } from "./helpers";
import { byId, type Input } from "./query";

export function throughImport(input: Input) {
  requireValue(input);
  return byId(read, input.id);
}

async function queryInput(query: SessionExecutor, input: Input) {
  const { id } = input;
  const rows = await query(sql`SELECT id FROM sample WHERE id = ${id}`);
  requireValue(input);
  return rows;
}

export function afterProjection(input: Input) {
  return queryInput(read, input);
}

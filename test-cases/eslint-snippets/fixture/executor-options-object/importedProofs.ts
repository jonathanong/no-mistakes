import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as resolvers from "./resolvers";

type Widget = { id: string; state: string };
type Result = { rows: Widget[] };

async function load(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  return rows[0];
}

export async function declaredCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectDeclared(id, { query });
  return row.state;
}

export async function optionalParameterCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectOptionalParameter(id, { query });
  return row.state;
}

export async function nullableCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectUnioned(id, { query });
  return row.state;
}

export async function defaultedCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectDefaulted(id, { query });
  return row.state;
}

export async function destructuredCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectDestructured(id, { query });
  return row.state;
}

import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as resolvers from "./resolvers";

type Widget = { id: string; state: string };
type Result = { rows: Widget[] };

async function load(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  return rows[0];
}

export async function untypedCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectUntyped(id, { query });
  return row.state;
}

export async function anyCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectAny(id, { query });
  return row.state;
}

export async function unknownCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectUnknown(id, { query });
  return row.state;
}

export async function functionCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectFunction(id, { query });
  return row.state;
}

export async function optionalPropertyCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectOptionalProperty(id, { query });
  return row.state;
}

export async function inferredCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectInferred(id, { query });
  return row.state;
}

export async function escapingCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectEscaping(id, { query });
  return row.state;
}

export async function unionCall(query: SessionExecutor, id: string) {
  const row = await load(query, id);
  if (!row) return resolvers.rejectUnion(id, { query });
  return row.state;
}

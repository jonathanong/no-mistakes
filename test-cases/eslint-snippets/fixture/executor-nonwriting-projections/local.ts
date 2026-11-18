import sql from "@example/sql";
import { read, type SessionExecutor } from "@example/db";

export type Input = { id: string; other: string };

function canEdit(input: Input) {
  return input.id !== "" && input.other !== "";
}

function byId(query: SessionExecutor, id: string) {
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}

export function beforeScalarHelper(input: Input) {
  if (canEdit(input)) return byId(read, input.id);
}

async function afterDestructuredRead(query: SessionExecutor, input: Input) {
  const { id } = input;
  const rows = await query(sql`SELECT id FROM sample WHERE id = ${id}`);
  canEdit(input);
  return rows;
}

export function withDestructuredRead(input: Input) {
  return afterDestructuredRead(read, input);
}

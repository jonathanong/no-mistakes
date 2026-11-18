import sql from "@example/sql";
import type { SessionExecutor } from "@example/db";
import type { Input } from "./local";

export function canEdit(input: Input) {
  return input.id !== "" && input.other !== "";
}

export function byId<Row>(query: SessionExecutor, id: string): Promise<Row[]> {
  return query(sql`SELECT id FROM sample WHERE id = ${id}`) as Promise<Row[]>;
}

export function withInput(query: SessionExecutor, input: Input) {
  if (canEdit(input)) {
    return query(sql`SELECT id FROM sample WHERE id = ${input.id}`);
  }
}

export function afterDestructuredExecutor(query: SessionExecutor, input: Input) {
  const { id } = input;
  const rows = query(sql`SELECT id FROM sample WHERE id = ${id}`);
  canEdit(input);
  return rows;
}

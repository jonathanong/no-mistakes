import sql, { type SqlStatement } from "@example/sql";
import { read, type SessionExecutor } from "@example/db";
import { memberProjection, typedScalar, type Input } from "./local";

export function loopOf(input: Input, ids: string[]) {
  for (input.id of ids) {}
  return memberProjection(read, input);
}
export function loopIn(input: Input, ids: Record<string, string>) {
  for (input.id in ids) {}
  return memberProjection(read, input);
}
function constructorProjection(query: SessionExecutor, value: string) {
  return query(sql`SELECT ${value.constructor}`);
}
export function scalarProperty(input: Input) {
  return constructorProjection(read, input.id);
}
export class PrivateInput {
  id: string = "public";
  #id: SqlStatement = sql`SELECT 1`;
  static scalar(query: SessionExecutor, input: PrivateInput) {
    return typedScalar(query, input.#id);
  }
  static caller(input: PrivateInput) {
    function projection(query: SessionExecutor, value: PrivateInput) {
      return query(sql`SELECT ${value.#id}`);
    }
    return projection(read, input);
  }
}

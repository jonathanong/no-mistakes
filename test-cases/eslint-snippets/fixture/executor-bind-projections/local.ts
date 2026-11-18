import sql from "@example/sql";
import { read, write, type SessionExecutor } from "@example/db";

export type Input = { id: string };
export interface Fields extends Input {
  count: number;
  enabled: boolean;
  total: bigint;
  nullable: string | null;
  kind: "basic" | "full";
}
export async function fixedProjection(query: SessionExecutor | typeof write, input: Input) {
  const { id } = input;
  await query(sql`SELECT count(*) FROM sample WHERE id = ${id}`);
}
export function renamedProjection(query: SessionExecutor, input: Input) {
  const { id: bound } = input;
  return query(sql`SELECT count(*) FROM sample WHERE id = ${bound}`);
}
export function parameterProjection(query: SessionExecutor, { id }: Input) {
  return query(sql`SELECT count(*) FROM sample WHERE id = ${id}`);
}
export function memberProjection(query: SessionExecutor, input: Input) {
  return query(sql`SELECT count(*) FROM sample WHERE id = ${input.id}`);
}
export function primitiveFields(query: SessionExecutor, input: Fields) {
  return query(sql`SELECT id FROM sample WHERE id = ${input.id}
    AND count = ${input.count} AND enabled = ${input.enabled}
    AND total = ${input.total} AND nullable = ${input.nullable} AND kind = ${input.kind}`);
}
export function typedScalar(query: SessionExecutor, id: string) {
  return query(sql`SELECT count(*) FROM sample WHERE id = ${id}`);
}
export function withObject(input: Input) {
  return fixedProjection(write, input);
}
export function withLiteral() {
  return fixedProjection(write, { id: "sample" });
}
export function withRename(input: Input) {
  return renamedProjection(read, input);
}
export function withParameter(input: Input) {
  return parameterProjection(read, input);
}
export function withMember(input: Input) {
  return memberProjection(read, input);
}
export function withFields(input: Fields) {
  return primitiveFields(read, input);
}
export function scalarMember(query: SessionExecutor, input: Input) {
  return typedScalar(query, input.id);
}
export function scalarParameter(query: SessionExecutor, id: string) {
  return typedScalar(query, id);
}

export function unionProjection(query: SessionExecutor, input: { id: string | number }) {
  return query(sql`SELECT id FROM sample WHERE id = ${input.id}`);
}
export function withUnion(input: Input | { id: number }) {
  return unionProjection(read, input);
}

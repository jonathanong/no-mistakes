import sql from "sql-template-strings";

export type Ref = { kind: "a"; a: string } | { kind: "b"; b: string };

export function where(ref: Ref) {
  if ("a" in ref) return sql`s.kind = ${ref.kind} AND s.a_id = ${ref.a}`;
  return sql`s.kind = ${ref.kind} AND s.b_id = ${ref.b}`;
}

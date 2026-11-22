import sql from "@example/sql";
import "./query-broken";

export function read(id: number) {
  return sql`select ${id}`;
}

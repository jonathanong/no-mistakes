import sql from "@example/sql";
import "./nested-query-broken";

export function createQuery() {
  return function query(id: number) {
    return sql`select ${id}`;
  };
}

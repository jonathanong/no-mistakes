import { query } from "@example/db";

export function settleBilling(assembledSql: string) {
  return query(assembledSql);
}

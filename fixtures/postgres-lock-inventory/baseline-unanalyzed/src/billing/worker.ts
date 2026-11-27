import { query } from "@example/db";

declare function lockAccountRow(id: string): Promise<void>;

export function settleBilling(id: string, assembledSql: string) {
  void lockAccountRow(id);
  return query(assembledSql);
}

import { type Query } from "db";
import { assertSafeSqlIdentifier } from "sqlid/partitions";
export async function partition(query: Query, tableName: string) {
  const partitionName = assertSafeSqlIdentifier(tableName);
  return query(`DELETE FROM items USING ${partitionName} AS chunk WHERE items.id = chunk.id`);
}

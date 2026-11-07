import { assertWhitelistedSqlIdentifier, type Query } from "db";
import { references } from "./references.ts";
const tableNames = references.map(([table]) => table);
export async function purgePage(query: Query, tableName: string, id: string) {
  const table = assertWhitelistedSqlIdentifier(tableName, tableNames, "purgeTable");
  await query(`/* purgePage */ DELETE FROM ${table} WHERE ref_id = $1::uuid`, [id]);
  await query(`DELETE FROM ${tableName} WHERE ref_id = $1::uuid`, [id]);
}

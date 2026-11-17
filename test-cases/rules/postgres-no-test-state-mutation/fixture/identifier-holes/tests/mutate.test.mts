import { query } from "db";
import sql from "sql-template-strings";
import { keyColumn, table } from "relation-names";

export async function rotate(v: string, rowId: string) {
  await query(
    sql`UPDATE ${table} SET token = ${v} WHERE ${keyColumn} = ${rowId}`,
  );
}

export async function remove(id: string) {
  await query(sql`DELETE FROM ${table} WHERE ${keyColumn} = ${id}`);
}

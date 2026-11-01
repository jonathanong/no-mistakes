import { beginTransaction } from "@example/db";

export async function plain(tableUnions: string, batchColumn: string, id: string) {
  await using query = await beginTransaction();
  await query(`SELECT id FROM ${tableUnions} WHERE id = $1 ORDER BY ${batchColumn}`, [id]);
}

import sql from "@example/sql";
import { read, write, type SessionExecutor } from "@example/db";
import * as db from "@example/db";

export async function directWithOptions(query: SessionExecutor, id: string) {
  await write(sql`UPDATE sample SET enabled = true WHERE id = ${id}`, { query });
  await write("UPDATE sample SET enabled = true WHERE id = $1", [id], { query });
  await write(`UPDATE sample SET enabled = true WHERE id = $1`, [id], { query });
  await write(`UPDATE sample SET enabled = true WHERE id = '${id}'`, { query });
  await write("SELECT 1", { query: read });
  const execute = write;
  await execute("SELECT 1", { query });
  await db.write("SELECT 1", { query });
  await db["write"]("SELECT 1", { query });
  await write.call(null, "SELECT 1", { query });
}
export async function directWithoutOptions(id: string) {
  await write(sql`UPDATE sample SET enabled = true WHERE id = ${id}`);
}

export function assertedOptions(query: SessionExecutor) {
  return write("SELECT 1", { query } as { query: SessionExecutor });
}

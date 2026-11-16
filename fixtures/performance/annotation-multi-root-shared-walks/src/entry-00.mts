import { query } from "@bench/db";
import sql from "sql-template-strings";
import { provide } from "./shared/helper-3.mjs";
export async function read00(flag: boolean) {
  await query(sql`SELECT id FROM items`);
  return provide(flag);
}

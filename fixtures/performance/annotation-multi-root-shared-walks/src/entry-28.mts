import { query } from "@bench/db";
import sql from "sql-template-strings";
import { provide } from "./shared/helper-3.mjs";
export async function read28(flag: boolean) {
  return provide(flag);
}

import sql from "sql-template-strings";
import { read } from "@example/db";

const FRAG = sql`OFFSET 10`;

export function list() {
  return read(`SELECT id FROM orders ${FRAG}`);
}

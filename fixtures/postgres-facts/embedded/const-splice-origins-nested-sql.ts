import sql from "sql-template-strings";
import { read } from "@example/db";

// Same-file const text is inlined into the nested template. Origin bytes must
// follow that text; a shorter placeholder-sized list used to panic publish.
const C = "id, name, created_at";
const LONG = "id, name, created_at, updated_at, status, owner_id";

export const f = (id: string) =>
  read(sql`${sql(`SELECT ${C} FROM t`)} WHERE id = ${id}`);

export const g = (id: string) =>
  read(sql`${sql(`SELECT ${LONG} FROM t`)} WHERE id = ${id} AND other = ${id}`);

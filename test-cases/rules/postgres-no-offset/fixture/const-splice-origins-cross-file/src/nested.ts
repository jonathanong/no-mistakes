import sql from "sql-template-strings";
import { read } from "@example/db";

// This shape panics publish when origin bytes do not match the spliced SQL.
// The sibling file's OFFSET must still be reported.
const C = "id, name, created_at";

export const f = (id: string) =>
  read(sql`${sql(`SELECT ${C} FROM t`)} WHERE id = ${id}`);

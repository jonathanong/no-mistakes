import sql from "sql-template-strings";
import { read } from "@example/db";
const C = "id, name, created_at";
export const f = (id: string) =>
  read(`${`SELECT ${C} FROM t`} WHERE id = 1 OFFSET 5`);

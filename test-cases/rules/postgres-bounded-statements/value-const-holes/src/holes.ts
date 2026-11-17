import templateSql from "sql-template-strings";
import { query } from "@example/db";

export function adjacent(a: string, b: string, c: string, n: number) {
  return query(templateSql`SELECT (${a}, ${b}${c}) FROM widgets WHERE n = $${n}`);
}

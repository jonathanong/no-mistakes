import { default as tag } from "sql-template-strings";
import { query } from "@data-stores/psql";

export function load(id: number) {
  return query(tag`SELECT * FROM topics WHERE id = ${id}`);
}

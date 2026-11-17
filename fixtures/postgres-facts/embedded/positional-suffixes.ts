import SQL from "sql-template-strings";
import { query } from "@example/db";

export function run(index: number, value: string) {
  query(SQL`SELECT $${1.5}, ${value}, $${1e21}, $${"12"}, ${value}`);
  query(SQL`SELECT $${10}, ${value}, $${1e2}, $${index}, ${value}`);
  query(SQL`SELECT $${-1.5}, $${-1}, $${+1.5}, $${+10}, ${value}`);
}

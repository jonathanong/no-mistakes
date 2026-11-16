import SQL from "sql-template-strings";
import { query } from "@example/db";

export function sameSqlLine(first: number, second: number) {
  return query(SQL`SELECT ${
    first
  } AS first, ${
    second
  } AS second, * FROM entries OFFSET 1`);
}

export function followingSqlLine(first: number, second: number) {
  return query(SQL`SELECT ${first} AS first,
    ${
      second
    } AS second, * FROM entries OFFSET 2`);
}

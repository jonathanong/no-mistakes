import { query } from "@example/db";

export function unresolved(left: string, right: string) { query(left); query(right); }

export function alternatives(flag: boolean) {
  const malformed = "SELECT FROM ???;";
  return query(flag ? `${malformed} SELECT 1;` : `${malformed} SELECT 2;`);
}

import { query } from "@example/db";

export function broken() {
  return query(`not valid SQL`);
}

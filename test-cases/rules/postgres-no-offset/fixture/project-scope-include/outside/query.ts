import { query } from "@example/db";

// Same OFFSET call as inside/query.ts. A scan wider than the project include would report this file.
export function page() {
  return query(`SELECT id FROM posts ORDER BY id DESC OFFSET 10`);
}

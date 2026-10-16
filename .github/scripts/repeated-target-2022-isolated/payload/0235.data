import { query } from "@example/db";

const sql = () => "DROP TABLE users";

function build(id: number) {
  return sql`SELECT * FROM topics WHERE id = ${id}`;
}

export function run() {
  return query(build(1));
}

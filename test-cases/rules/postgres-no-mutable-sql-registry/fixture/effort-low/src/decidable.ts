import { query } from "@example/db";
// A value placeholder cannot mutate a SQL relation identifier.
export function literal(id: string) {
  query("SELECT id FROM app.users");
  query(`SELECT id FROM app.users`);
  query(`SELECT id FROM app.users WHERE id = ${id}`);
  const statement = "SELECT id FROM app.users";
  query(statement);
}

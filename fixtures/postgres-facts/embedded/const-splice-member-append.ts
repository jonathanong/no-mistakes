import sql from "sql-template-strings";
import { read } from "@example/db";

const TABLES = { users: "users" };

export function list() {
  const query = sql`SELECT id FROM `;
  query.append(TABLES.users);
  query.append(" WHERE account_id = $1");
  return read(query);
}

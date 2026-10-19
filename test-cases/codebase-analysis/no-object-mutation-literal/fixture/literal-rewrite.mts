// A spread of an existing object is not a known value. Declare the object once.
import { query } from "@example/db";

export const tables = { users: "app.users", orders: "app.orders" } as const;

export function replicaClient(get) {
  return { client: { get } };
}

query(`SELECT id FROM ${tables.users} WHERE active`);

import { query } from "@example/db";

const holder = {
  tables: new Map([
    ["user", "users"],
    ["post", "posts"],
  ]),
};

export function nested(kind: string) {
  return query(`SELECT id FROM ${holder.tables.get(kind)}`);
}

import { query } from "@example/db";

export function lockBroken() {
  return query(`SELECT FROM items FOR UPDATE (`);
}

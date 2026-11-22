import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read21() {
  invoke03(() => query("/* entry_21 */ SELECT 21"), payload);
  return query("SELECT id FROM items WHERE id = 21"); // expected-finding
}

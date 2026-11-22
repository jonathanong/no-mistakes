import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read24() {
  invoke03(() => query("/* entry_24 */ SELECT 24"), payload);
  return query("SELECT id FROM items WHERE id = 24"); // expected-finding
}

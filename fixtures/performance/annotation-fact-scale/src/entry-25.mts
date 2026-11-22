import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read25() {
  invoke03(() => query("/* entry_25 */ SELECT 25"), payload);
  return query("SELECT id FROM items WHERE id = 25"); // expected-finding
}

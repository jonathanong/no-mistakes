import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read10() {
  invoke03(() => query("/* entry_10 */ SELECT 10"), payload);
  return query("SELECT id FROM items WHERE id = 10"); // expected-finding
}

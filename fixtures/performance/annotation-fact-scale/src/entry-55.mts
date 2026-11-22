import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read55() {
  invoke03(() => query("/* entry_55 */ SELECT 55"), payload);
  return query("SELECT id FROM items WHERE id = 55"); // expected-finding
}

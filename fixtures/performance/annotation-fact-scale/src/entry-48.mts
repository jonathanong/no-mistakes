import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read48() {
  invoke03(() => query("/* entry_48 */ SELECT 48"), payload);
  return query("SELECT id FROM items WHERE id = 48"); // expected-finding
}

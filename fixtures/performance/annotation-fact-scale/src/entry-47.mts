import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read47() {
  invoke03(() => query("/* entry_47 */ SELECT 47"), payload);
  return query("SELECT id FROM items WHERE id = 47"); // expected-finding
}

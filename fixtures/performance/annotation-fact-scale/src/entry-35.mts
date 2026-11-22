import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read35() {
  invoke03(() => query("/* entry_35 */ SELECT 35"), payload);
  return query("SELECT id FROM items WHERE id = 35"); // expected-finding
}

import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read08() {
  invoke03(() => query("/* entry_08 */ SELECT 8"), payload);
  return query("SELECT id FROM items WHERE id = 8"); // expected-finding
}

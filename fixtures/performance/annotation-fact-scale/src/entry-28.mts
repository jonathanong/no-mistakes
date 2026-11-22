import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read28() {
  invoke03(() => query("/* entry_28 */ SELECT 28"), payload);
  return query("SELECT id FROM items WHERE id = 28"); // expected-finding
}

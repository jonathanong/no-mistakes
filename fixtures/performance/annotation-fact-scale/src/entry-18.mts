import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read18() {
  invoke03(() => query("/* entry_18 */ SELECT 18"), payload);
  return query("SELECT id FROM items WHERE id = 18"); // expected-finding
}

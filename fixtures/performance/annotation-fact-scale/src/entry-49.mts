import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read49() {
  invoke03(() => query("/* entry_49 */ SELECT 49"), payload);
  return query("SELECT id FROM items WHERE id = 49"); // expected-finding
}

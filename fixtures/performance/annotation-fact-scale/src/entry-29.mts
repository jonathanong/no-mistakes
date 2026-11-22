import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read29() {
  invoke03(() => query("/* entry_29 */ SELECT 29"), payload);
  return query("SELECT id FROM items WHERE id = 29"); // expected-finding
}

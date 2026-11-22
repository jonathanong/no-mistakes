import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read42() {
  invoke03(() => query("/* entry_42 */ SELECT 42"), payload);
  return query("SELECT id FROM items WHERE id = 42"); // expected-finding
}

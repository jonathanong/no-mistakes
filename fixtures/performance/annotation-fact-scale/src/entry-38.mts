import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read38() {
  invoke03(() => query("/* entry_38 */ SELECT 38"), payload);
  return query("SELECT id FROM items WHERE id = 38"); // expected-finding
}

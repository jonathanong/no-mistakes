import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read20() {
  invoke03(() => query("/* entry_20 */ SELECT 20"), payload);
  return query("SELECT id FROM items WHERE id = 20"); // expected-finding
}

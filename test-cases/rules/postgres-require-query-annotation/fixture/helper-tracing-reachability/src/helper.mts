import { write } from "@app/db";
export function run() {
  write("SELECT 1"); // finding:reachable-imported-executor
}

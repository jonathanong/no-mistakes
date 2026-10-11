import { write } from "@app/db";
export function run(sql: string) {
  write(sql); // finding:reachable-imported-executor
}

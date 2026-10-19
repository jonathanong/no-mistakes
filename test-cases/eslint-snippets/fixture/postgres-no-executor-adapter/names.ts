import { write } from "@example/db";

declare function run(exec: unknown, account: string): unknown;

export function load(account: string) {
  run(write, account);
}

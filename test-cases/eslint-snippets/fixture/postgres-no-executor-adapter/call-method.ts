import * as db from "@example/db";

declare function run(exec: (statement: string) => unknown, account?: string): unknown;

// `call` is the configured executor, not Function.prototype.call.
// The statement stays the first argument.
export function load(account: string) {
  run((statement) => db.call(statement), account);
  run((statement) => db["call"](statement), account);
}

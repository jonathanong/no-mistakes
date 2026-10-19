import { read } from "@example/db";

declare function run(exec: unknown, account: string): unknown;

export function load(account: string) {
  // no-mistakes-disable-next-line postgres-no-executor-adapter -- reviewed forwarder
  run(read.bind(null), account);
  run(read, account);
}

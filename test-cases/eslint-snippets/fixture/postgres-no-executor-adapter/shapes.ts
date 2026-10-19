import { read, write } from "@example/db";

declare function run(exec: (statement: string) => unknown, account: string): unknown;
declare class Runner {
  constructor(exec: (statement: string) => unknown);
}

export function list(account: string, flag: boolean) {
  function adapter(statement: string) {
    return read(statement);
  }
  run(adapter, account);
  new Runner(read);
  new Runner((statement: string) => read(statement));
  run({ exec: read }, account);
  run([read], account);
  run(read.bind(null), account);
  run(read ?? write, account);
  run(flag && read, account);
}

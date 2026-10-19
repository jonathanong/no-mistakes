import { openTx, type Tx } from "@example/db";

declare function run(exec: unknown, account?: string): unknown;
declare class Runner {
  constructor(exec: unknown);
}

export function load(account: string, tx: Tx) {
  run(tx.query.bind(null), account);
  const alias = tx;
  new Runner(alias.query);
  run(openTx().query, account);
}

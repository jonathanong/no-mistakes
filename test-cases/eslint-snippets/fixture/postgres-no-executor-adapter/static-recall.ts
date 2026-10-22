import { read, type Tx } from "@example/db";
import * as db from "@example/db";

declare function run(exec: (statement: string) => unknown, account?: string): unknown;

// Namespace members, a const query destructure, and optional or static
// `.call` / `.apply` / `.bind` are the same executor as the dotted form.
export function namespace(account: string) {
  run((statement) => db.read(statement), account);
  run((statement) => db["read"](statement), account);
  run((statement) => db[`read`](statement), account);
  run((statement) => db?.read(statement), account);
  run((statement) => (db as object).read(statement), account);
  run(db.read, account);
  run(db?.read, account);
}

export function destructured(tx: Tx, account: string, flag: boolean) {
  const { query } = tx;
  run((statement) => query(statement), account);
  const { query: renamed } = tx;
  run((statement) => renamed(statement), account);
  const { ["query"]: bracket } = tx;
  run((statement) => bracket(statement), account);
  const { [`query`]: template } = tx;
  run((statement) => template(statement), account);
  // prettier-ignore
  const { "query": quoted } = tx;
  run((statement) => quoted(statement), account);
  const { query: asserted } = tx as Tx;
  run((statement) => asserted(statement), account);
  const { read: fromNamespace } = db;
  run((statement) => fromNamespace(statement), account);
  run((statement) => tx?.query(statement), account);
  run(flag && read?.bind(null), account);
}

export function wrappers(account: string) {
  run((statement) => read?.(statement), account);
  run((statement) => read["call"](null, statement), account);
  run((statement) => read["apply"](null, [statement]), account);
  run((statement) => read[`call`](null, statement), account);
  run((statement) => read[`apply`](null, [statement]), account);
  run((statement) => read?.call(null, statement), account);
  run((statement) => read?.["apply"](null, [statement]), account);
  run(read?.bind(null), account);
  run(read.bind?.(null), account);
  run(read["bind"](null), account);
  run(read[`bind`](null), account);
}

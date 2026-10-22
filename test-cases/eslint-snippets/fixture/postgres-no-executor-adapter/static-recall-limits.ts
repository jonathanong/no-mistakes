import { read, type Tx } from "@example/db";
import * as db from "@example/db";
import * as other from "@other/db";
import * as sub from "@example/db/read";
import type * as types from "@example/db";
import reader from "@example/db";

declare function run(exec: (statement: string) => unknown, account?: string): unknown;

// These stay silent: another module, a subpath, a type-only namespace, a
// default import, a computed identifier, a destructure default, `let`, an
// unproven receiver, and `.call` / `.bind` that are not the executor.
export function limits(
  tx: Tx,
  account: string,
  name: string,
  client: { query(sql: string): unknown },
  fallback: (sql: string) => unknown,
) {
  run((statement) => other.read(statement), account);
  run((statement) => sub.read(statement), account);
  run((statement) => types.read(statement), account);
  run((statement) => reader(statement), account);
  run((statement) => db[name](statement), account);
  run((statement) => db[`${name}`](statement), account);
  run((statement) => db[1](statement), account);
  run((statement) => db.other(statement), account);
  run((statement) => undeclared.read(statement), account);
  run((statement) => undeclared().read(statement), account);
  run((statement) => undeclared()(statement), account);
  function helper(value: unknown) {
    return value;
  }
  run((statement) => helper.read(statement), account);
  run((statement) => helper.call(null, statement), account);

  let { query: live } = tx;
  run((statement) => live(statement), account);
  const { query = fallback } = tx;
  run((statement) => query(statement), account);
  const { ...rest } = tx;
  run((statement) => rest(statement), account);
  const { other: extra } = tx;
  run((statement) => extra(statement), account);
  const { query: untyped } = client;
  run((statement) => untyped(statement), account);
  const { [name]: computed } = tx;
  run((statement) => computed(statement), account);
  const { [`q${name}`]: templated } = tx;
  run((statement) => templated(statement), account);
  const { 1: numbered } = tx;
  run((statement) => numbered(statement), account);
  const { query: member } = tx.query;
  run((statement) => member(statement), account);
  const { missingExport: picked } = db;
  run((statement) => picked(statement), account);
  const local = (sql: string) => sql;
  run((statement) => local(statement), account);
  run((statement) => statement(statement), account);
  run((statement) => read[name](null, statement), account);
  run((statement) => read[`${name}`](null, statement), account);
  run(read?.bind(null, account), account);
  run(read.bind, account);
}

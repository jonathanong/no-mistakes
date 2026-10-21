import { read } from "@example/db";
const TABLES = { orders: "orders" };

export function callSuppressed(kind: string) {
  const statement = `SELECT ${TABLES[kind]}`;
  // no-mistakes-disable-next-line postgres-no-nonliteral-sql-registry-key -- reviewed call
  read(statement);
}
export function lookupSuppressed(kind: string) {
  // no-mistakes-disable-next-line postgres-no-nonliteral-sql-registry-key -- reviewed lookup
  const statement = `SELECT ${TABLES[kind]}`;
  read(statement);
}
export function sharedInitializer(kind: string) {
  const statement = `SELECT ${TABLES[kind]}`;
  // One suppressed call must not hide an independently executed lookup.
  // no-mistakes-disable-next-line postgres-no-nonliteral-sql-registry-key -- reviewed call
  read(statement);
  read(statement);
}
export function multilineCall(kind: string) {
  // no-mistakes-disable-next-line postgres-no-nonliteral-sql-registry-key -- reviewed call
  read(
    `SELECT ${TABLES[kind]}`,
  );
}

import { read } from "@example/db";

const TABLES = { orders: "orders" };

export function kept(kind: string) {
  // no-mistakes-disable-next-line postgres-no-nonliteral-sql-registry-key -- reviewed registry
  read(`SELECT ${TABLES[kind]}`);
  read(`SELECT ${TABLES[kind]}`);
}

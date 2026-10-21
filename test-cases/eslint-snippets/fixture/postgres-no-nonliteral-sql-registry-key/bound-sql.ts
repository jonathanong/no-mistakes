import { read } from "@example/db";
const TABLES = { orders: "orders", accounts: "accounts" };
export function byKind(kind: string) {
  const query = `SELECT ${TABLES[kind]}`;
  read(query);
  // Reuse should not duplicate a finding at its declaration.
  read(query);
  const tagged = sql`SELECT ${TABLES[kind]}`;
  read(tagged);
}
declare function sql(text: TemplateStringsArray, ...values: unknown[]): string;

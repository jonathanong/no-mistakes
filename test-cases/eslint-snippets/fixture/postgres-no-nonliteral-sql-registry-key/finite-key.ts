import { read } from "@example/db";
const TABLES = { orders: "orders", accounts: "accounts" };
export function matching(kind: string) {
  // This explicit finite-key contract refers to this exact registry binding.
  read(`SELECT ${TABLES[kind as keyof typeof TABLES]}`);
  // An empty string is still a literal key, regardless of whether it exists.
  read(`SELECT ${TABLES[""]}`);
}

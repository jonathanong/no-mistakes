import { read } from "@example/db";
import * as sql from "sql-template-strings";
export function namespaceTag(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = sql`SELECT 2`;
  }
}

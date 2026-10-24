// no-mistakes-disable-file postgres-require-query-annotation
import { read } from "@example/db";
import sql from "sql-template-strings";
export function suppressed(xs: readonly unknown[]) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    read(q);
    q = sql`SELECT 2`;
  }
}

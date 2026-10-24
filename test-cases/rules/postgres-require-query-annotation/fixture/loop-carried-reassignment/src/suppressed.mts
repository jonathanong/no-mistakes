import { read } from "@example/db";
import sql from "sql-template-strings";
export function suppressed(xs: readonly unknown[]) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    // no-mistakes-disable-next-line postgres-require-query-annotation
    read(q);
    q = sql`SELECT 2`;
  }
}
export function lineSuppressed(xs: readonly unknown[]) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    read(q); // no-mistakes-disable-line postgres-require-query-annotation
    q = sql`SELECT 2`;
  }
}

import { read } from "@example/db";
import { statement as build } from "@custom/sql";
export function configured(xs: readonly unknown[]) {
  let q = build`/* named */ SELECT 1`;
  for (const x of xs) {
    read(q);
    q = build`SELECT 2`;
  }
}

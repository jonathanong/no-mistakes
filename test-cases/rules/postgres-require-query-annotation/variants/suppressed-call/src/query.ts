import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-require-query-annotation
query(flag
  ? "SELECT 1"
  : "/* items/read */ SELECT 1");

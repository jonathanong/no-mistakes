import { query } from "@example/db";
query(flag
  // no-mistakes-disable-next-line postgres-require-query-annotation
  ? "SELECT 1"
  : "/* users/read */ SELECT 1");

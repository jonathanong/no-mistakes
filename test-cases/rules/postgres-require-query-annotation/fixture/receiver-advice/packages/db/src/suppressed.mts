import { writePool } from "./db.mjs";

// no-mistakes-disable-next-line postgres-require-query-annotation
writePool.query("SELECT 2");

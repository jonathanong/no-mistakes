import SQL from "sql-template-strings";
import { query } from "@example/db";
import { discoverable } from "./discoverable";

export function list(batchId: string) {
  return query(
    // no-mistakes-disable-next-line postgres-required-predicates
    SQL`SELECT c.id FROM candidates c WHERE c.batch_id = ${batchId} AND `.append(discoverable("c.id")).append(SQL` ORDER BY c.id LIMIT 50`),
  );
}

export function listUnsuppressed(batchId: string) {
  return query(
    SQL`SELECT c.id FROM candidates c WHERE c.batch_id = ${batchId} AND `.append(discoverable("c.id")).append(SQL` ORDER BY c.id LIMIT 50`),
  );
}

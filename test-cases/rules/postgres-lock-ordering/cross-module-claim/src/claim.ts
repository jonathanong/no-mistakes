import SQL from "sql-template-strings";
import { query } from "@example/db";
import { TIMEOUT_MS } from "./limits";
import { claimable } from "./policy.mts";

export function claimWork(ids: string[], now: Date) {
  return query(
    SQL`/* claimWork */ WITH candidate AS (SELECT work.id FROM work JOIN records record ON record.id = work.record_id WHERE work.id = ANY(${[...ids]}::uuid[]) AND work.generation = record.generation AND `
      .append(claimable(now))
      .append(SQL` ORDER BY work.id FOR UPDATE OF work ), claimed AS (UPDATE work SET lease_token = gen_random_uuid(), lease_expires_at = now() + ${TIMEOUT_MS} * interval '1 millisecond' FROM candidate WHERE work.id = candidate.id RETURNING work.id) SELECT id FROM claimed`),
  );
}

export function noncanonicalControl(ids: string[]) {
  return query(SQL`SELECT work.id FROM work WHERE work.id = ANY(${ids}::uuid[]) ORDER BY work.record_id FOR UPDATE OF work`);
}

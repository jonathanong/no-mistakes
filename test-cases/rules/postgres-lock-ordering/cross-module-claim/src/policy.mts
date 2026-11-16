import SQL from "sql-template-strings";
import { MAX_ATTEMPTS } from "./limits";

export function claimable(now: Date) {
  return SQL`failed_id IS NULL AND attempt_count < ${MAX_ATTEMPTS} AND ((lease_token IS NULL AND available_at <= GREATEST(${now}::timestamptz, clock_timestamp())) OR (lease_token IS NOT NULL AND lease_expires_at <= ${now}))`;
}

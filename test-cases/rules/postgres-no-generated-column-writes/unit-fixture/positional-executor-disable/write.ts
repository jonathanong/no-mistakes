import { query } from "@example/db";

export function insertVote() {
  // no-mistakes-disable-next-line postgres-no-generated-column-writes
  return query(
    flag
      ? "INSERT INTO votes VALUES ($1, $2)"
      : "INSERT INTO votes VALUES ($1)",
  );
}

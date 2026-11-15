import { query } from "@example/db";

export function insertVote() {
  return query(
    flag
      ? "INSERT INTO votes VALUES ($1, $2)"
      : "INSERT INTO votes VALUES ($1)",
  );
}

import * as pool from "@example/db";

export function lockJobs(ids: string[]) {
  const parameters = [ids];
  const text = `SELECT id FROM jobs WHERE id = ANY($${1}::uuid[]) FOR UPDATE`;
  return pool.query(text, parameters);
}

import { query } from "@example/db";

// The local invocation is known, but external callers of this export are not.
export function run(sql: string) {
  query(sql);
}

run("SELECT id FROM widgets");

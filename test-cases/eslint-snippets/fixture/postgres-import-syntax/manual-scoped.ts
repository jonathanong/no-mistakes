import type { Db } from "./db";

function bad1({ ["db" as const]: db }: { db: Db }) {
  db("BEGIN");
}

function bad2(deps: { db: Db }) {
  deps!.db("BEGIN");
  (deps as { db: Db }).db("BEGIN");
}

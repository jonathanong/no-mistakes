import type { Db } from "./db";

function run({ read, [`extra`]: extra, ...rest }: { read: Db; extra: Db }) {
  read("BEGIN");
  extra("BEGIN");
  return rest;
}

declare const slot: "actual";

function computed(db: { [slot]: Db }) {
  db.slot("BEGIN");
}

function literalKey(db: { ["read"]: Db }) {
  db.read("BEGIN");
}

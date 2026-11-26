import type { Db } from "./db";

function run({ read, [`extra`]: extra, ...rest }: { read: Db; extra: Db }) {
  read("BEGIN");
  extra("BEGIN");
  return rest;
}

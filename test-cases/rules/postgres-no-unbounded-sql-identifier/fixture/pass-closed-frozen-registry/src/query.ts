import { query } from "@example/db";

const LOCAL_FROZEN = Object.freeze({
  user: "users",
  post: "posts",
});

export function frozenMember() {
  return query(`SELECT id FROM ${LOCAL_FROZEN.user}`);
}

export function copiedMember() {
  const entity = LOCAL_FROZEN.user;
  return query(`SELECT id FROM ${entity}`);
}

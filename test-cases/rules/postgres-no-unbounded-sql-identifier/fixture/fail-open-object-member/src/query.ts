import { query } from "@example/db";

const OPEN = {
  user: "users",
  extra: process.env.EXTRA_TABLE,
};

export function openMember() {
  return query(`SELECT id FROM ${OPEN.extra}`);
}

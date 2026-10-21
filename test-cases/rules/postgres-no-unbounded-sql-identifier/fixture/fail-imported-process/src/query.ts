import { query } from "@example/db";
import { process } from "node:process";

export function list() {
  return query(`SELECT id FROM ${process.env.SCHEMA}.users`);
}

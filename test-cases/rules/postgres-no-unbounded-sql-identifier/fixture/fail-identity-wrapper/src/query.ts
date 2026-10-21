import { query } from "@example/db";

const identity = (value: string) => value;

export function list() {
  return query(`SELECT id FROM ${identity(process.env.SCHEMA ?? "")}.users`);
}

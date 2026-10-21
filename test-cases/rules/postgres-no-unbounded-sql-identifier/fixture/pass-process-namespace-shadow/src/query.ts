import { query } from "@example/db";
import * as proc from "node:process";
function run() {
  const proc = { env: { SCHEMA: "app" } }; // The local finite object shadows the namespace import.
  query(`SELECT id FROM ${proc.env.SCHEMA}.users`);
}
run();

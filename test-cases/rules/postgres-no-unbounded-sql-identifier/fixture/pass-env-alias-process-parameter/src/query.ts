import { query } from "@example/db";
function run(process: { env: { SCHEMA: string } }) {
  const e = process.env; // This parameter is unrelated to the global process object.
  query(`SELECT id FROM ${e.SCHEMA}.users`);
}
run({ env: { SCHEMA: "app" } });

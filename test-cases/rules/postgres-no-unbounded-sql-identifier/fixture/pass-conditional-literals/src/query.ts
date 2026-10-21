import { query } from "@example/db";
// Env controls the choice, but neither VALUE branch contains env data.
query(`SELECT id FROM ${process.env.FLAG ? "app" : "public"}.users`);

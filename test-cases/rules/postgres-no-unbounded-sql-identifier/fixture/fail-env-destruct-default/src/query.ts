import { query } from "@example/db";
const { SCHEMA: schema = "app" } = process.env;
query(`SELECT id FROM ${schema}.users`);

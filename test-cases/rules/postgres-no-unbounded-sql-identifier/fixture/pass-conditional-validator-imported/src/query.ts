import { query } from "@example/db";
import { schemaOf } from "./schema";
query(`SELECT id FROM ${schemaOf(process.env.SCHEMA)}.users`);

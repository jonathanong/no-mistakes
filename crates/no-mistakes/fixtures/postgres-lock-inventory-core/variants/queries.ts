import sql from "sql-template-strings";
import { query } from "@example/db";

const complete = sql`SELECT 'é'`;
if (choice) complete.append(" /* branch */");
complete.append(" FROM jobs FOR SHARE");
void query(complete);

let sampled = sql`SELECT id FROM sampled_jobs FOR UPDATE`;
for (const value of inputs) sampled.append(" /* sampled */");
void query(sampled);

let opaque = sql`SELECT id FROM opaque_jobs FOR UPDATE`;
while (condition) {}
void query(opaque);

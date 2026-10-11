import { query, sql } from "@example/db";
query(sql`CREATE INDEX accounts_email ON accounts (email) ${flag ? sql`/* first */` : sql`/* second */`}`);

import sql from "sql-template-strings";
import { query } from "@example/db";
query(`/* template */ SELECT ${id}`);
query(String.raw`/* raw */ SELECT \n ${id}`);
query(sql`/* tag */ SELECT ${id}`);
query(sql`/* method */ SELECT 1`.untrusted("ignored"));
query(factory());
const q = sql`/* binding */ SELECT 1`;
query(q, change(q));

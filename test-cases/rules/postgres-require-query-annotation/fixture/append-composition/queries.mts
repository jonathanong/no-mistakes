import { read } from "@bench/db";
import sql from "sql-template-strings";
const query = sql`SELECT id FROM widgets WHERE TRUE`;
if (condition) query.append(" AND");
read(query);
const named = sql`/* named */ SELECT id FROM widgets WHERE TRUE`;
if (condition) named.append(" AND");
read(named);
const invalid = sql`SELECT (`;
if (condition) invalid.append(" AND");
read(invalid);
const bound = sql`/* bound */`.append(sql`SELECT id FROM widgets WHERE owner_id = ${ownerId}`);
read(bound);
read(sql`/* direct */`.append(sql`SELECT id FROM widgets WHERE owner_id = ${ownerId}`));
// no-mistakes-disable-next-line postgres-require-query-annotation
read(named);

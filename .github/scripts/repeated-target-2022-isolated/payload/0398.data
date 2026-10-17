import { query, sql } from "@example/db";
// Opaque falsy operands may be nullish. Publish no partial versions for ??.
query(sql`SELECT 1 ${(undefined && sql`LIMIT 1`) ?? sql`OFFSET 1`}`);
query(sql`SELECT 2 ${(nullableFlag && sql`LIMIT 1`) ?? sql`OFFSET 2`}`);

// Without ??, unknown falsy values all contribute the same absent fragment.
query(sql`SELECT 3 ${nullableFlag && sql`LIMIT 1`}`);
query(sql`SELECT 4 ${(nullableFlag && sql`LIMIT 1`) || sql`LIMIT 2`}`);

// Proven non-nullish falsy operands retain their existing absence behavior.
query(sql`SELECT 5 ${(false && sql`LIMIT 1`) ?? sql`OFFSET 5`}`);
query(sql`SELECT 6 ${(0 && sql`LIMIT 1`) ?? sql`OFFSET 6`}`);
query(sql`SELECT 7 ${("" && sql`LIMIT 1`) ?? sql`OFFSET 7`}`);

let statement = sql`SELECT 8`;
// An unproved nullable guard cannot discard effects in the ?? fallback.
(nullableFlag && sql`unused`) ?? statement.append(sql` OFFSET 8`);
query(statement);

const nullablePart = nullableFlag && sql`LIMIT 1`;
query(sql`SELECT 9 ${nullablePart ?? sql`OFFSET 9`}`);

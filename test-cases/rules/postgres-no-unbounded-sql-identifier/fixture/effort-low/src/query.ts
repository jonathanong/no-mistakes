import { query } from "@example/db";

query("SELECT id FROM public.users"); // clear: direct static SQL
const statement = "SELECT id FROM public.users";
query(statement); // clear: one immutable lexical const
query(`SELECT id FROM ${process.env.SCHEMA}.users`); // violation: direct env relation
query("SELECT id FROM public.users WHERE id = $1", [process.env.USER_ID]); // clear: bound value

declare function sqlHelper(): string;
query(sqlHelper()); // unsupported: helper-produced SQL
const alias = statement;
query(alias); // unsupported: two-hop const alias

// The lexical binding must win over the imported executor's spelling.
function unrelated(query: (sql: string) => void) {
  query(sqlHelper()); // unrelated: shadowed executor
}
unrelated(() => {});

// The global spelling is not proof when process is lexically shadowed.
function localProcess(process: { env: { SCHEMA: string } }) {
  query(`SELECT id FROM ${process.env.SCHEMA}.users`); // unsupported: shadowed process
}
localProcess({ env: { SCHEMA: "public" } });

// no-mistakes-disable-next-line postgres-no-unbounded-sql-identifier
query(sqlHelper()); // suppressed

query(`SELECT * FROM ${process.env.SCHEMA}.${process.env.TABLE}`); // violation: two distinct sources
query(`SELECT * FROM ${process.env.SCHEMA}.users`); query(`SELECT * FROM ${process.env.TABLE}.users`); // violation: separate same-line calls
query(`SELECT * FROM no_mistakes_low_hole_0 WHERE id = ${process.env.ID}`); // clear: collision cannot turn value into relation
query("SELECT * FROM sql_placeholder_0"); // clear: literal reserved-looking relation

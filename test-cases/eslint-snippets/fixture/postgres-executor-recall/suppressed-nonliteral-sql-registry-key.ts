import { query } from "@example/db";
const alias = query;
const TABLES = { orders: "orders" }; const kind = input;
// no-mistakes-disable-next-line postgres-no-nonliteral-sql-registry-key
alias(`SELECT ${TABLES[kind]}`);
alias(`SELECT ${TABLES[kind]}`); // no-mistakes-disable-line postgres-no-nonliteral-sql-registry-key

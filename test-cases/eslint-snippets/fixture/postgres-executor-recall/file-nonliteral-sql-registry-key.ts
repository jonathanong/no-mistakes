// no-mistakes-disable-file postgres-no-nonliteral-sql-registry-key
import { query } from "@example/db";
const alias = query;
const TABLES = { orders: "orders" }; const kind = input;
alias(`SELECT ${TABLES[kind]}`);

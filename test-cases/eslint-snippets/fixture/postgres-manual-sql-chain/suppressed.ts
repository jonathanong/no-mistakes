import { query } from "@example/db";
const begin = "BEGIN";
const alias = begin;
// no-mistakes-disable-next-line postgres-no-manual-transaction
query(alias);
query(alias); // no-mistakes-disable-line postgres-no-manual-transaction
query(alias);

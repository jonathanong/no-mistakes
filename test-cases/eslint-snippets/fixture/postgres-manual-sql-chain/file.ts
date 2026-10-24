// no-mistakes-disable-file postgres-no-manual-transaction
import { query } from "@example/db";
const begin = "BEGIN";
const alias = begin;
query(alias);

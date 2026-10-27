// no-mistakes-disable-file postgres-explicit-columns
// no-mistakes-disable-file postgres-required-predicates
import { query } from "@example/db";
query("SELECT * FROM accounts");

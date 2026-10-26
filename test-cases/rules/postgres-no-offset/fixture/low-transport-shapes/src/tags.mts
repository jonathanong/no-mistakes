import { read } from "@example/db";
import sql from "sql-template-strings";
declare const makeTag: () => unknown;
// Valid tagged-template syntax can have no cooked value; it cannot prove SQL.
read(sql`SELECT '\u{110000}'`); // ADVISE
read(makeTag()`SELECT 1`); // ADVISE

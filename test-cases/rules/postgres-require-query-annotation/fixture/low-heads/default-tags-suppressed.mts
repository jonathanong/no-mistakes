import { read } from "@bench/db";
import SQL from "sql-template-strings";
// no-mistakes-disable-next-line postgres-require-query-annotation
read(SQL`SELECT ${value}`);
read(SQL`SELECT ${value}`); // no-mistakes-disable-line postgres-require-query-annotation

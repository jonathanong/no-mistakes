import { read } from "@bench/db";
import sql from "any-builder";
// no-mistakes-disable-next-line postgres-require-query-annotation
read(sql`SELECT unnamed`.append(" suffix"));
read(sql`SELECT unnamed`.append(" suffix")); // no-mistakes-disable-line postgres-require-query-annotation

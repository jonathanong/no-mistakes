// no-mistakes-disable-file postgres-require-query-annotation
import { read } from "@bench/db";
import sql from "any-builder";
read(sql`SELECT unnamed`.append(" suffix"));

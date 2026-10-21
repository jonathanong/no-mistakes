import { query } from "@example/db";
declare const opaque: string;

// no-mistakes-disable-next-line postgres-require-query-annotation -- accepted opaque call
query(opaque);

// no-mistakes-disable-file postgres-require-query-annotation -- accepted opaque source
import { query } from "@example/db";
declare const opaque: string;

query(opaque);

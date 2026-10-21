import { query } from "@example/db";
declare const opaque: string;

query("SELECT * FROM orders");
query(opaque); // Each application owns an advisory obligation for this executor.
// The shared suppression pass audits one finding despite coincident applications.
// no-mistakes-disable-next-line postgres-explicit-columns -- fixture suppression parity
query("SELECT * FROM orders");

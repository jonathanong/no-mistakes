import { query } from "@example/db";
declare const opaque: string;
declare const http: { query(sql: string): void };

query("/* known */ SELECT id FROM widgets");
query(opaque); // Proven executor: residual leading SQL needs an advisory.
http.query(opaque); // Unowned receiver: no SQL candidate or advisory.

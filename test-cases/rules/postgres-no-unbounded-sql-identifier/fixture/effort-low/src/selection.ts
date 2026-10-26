import * as db from "@example/db";
import { query as offTarget } from "@example/db/other";
import { createQuery } from "@example/db";

// Namespace escape must not silently lose the eventual executor.
const run = db.query; // clear: transferring the namespace member does not execute SQL.
run("SELECT * FROM public.users"); // unsupported: the actual alias call retains unknown SQL.
db["query"]("SELECT * FROM public.users"); // unsupported: namespace call
const key = "query";
db[key]("SELECT * FROM public.users"); // unsupported
db.other("not SQL"); // unrelated
const unrelated = db.other; // unrelated
offTarget(`SELECT * FROM ${process.env.SCHEMA}.users`); // unrelated exact import scope

import * as sub from "@example/db/other";
sub.query("SELECT * FROM public.users"); // unsupported: historical namespace subpath selection

db.read(`SELECT * FROM ${process.env.SCHEMA}.users`); // unrelated namespace non-query member
db.write(`SELECT * FROM ${process.env.SCHEMA}.users`); // unrelated namespace non-query member

declare function unknownHelper(value: unknown): void;
unknownHelper(db); // unsupported: namespace escape
// No namespace history evaluator runs: the later call also remains unknown.
db.query(`SELECT * FROM ${process.env.SCHEMA}.users`); // unsupported: escaped namespace

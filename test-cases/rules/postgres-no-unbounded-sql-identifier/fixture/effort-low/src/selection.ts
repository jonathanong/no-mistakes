import * as db from "@example/db";
import { query as offTarget } from "@example/db/other";
import { createQuery } from "@example/db";

// Namespace escape must not silently lose the eventual executor.
const run = db.query; // unsupported
run("SELECT * FROM public.users");
db["query"]("SELECT * FROM public.users"); // clear
const key = "query";
db[key]("SELECT * FROM public.users"); // unsupported
db.other("not SQL"); // unrelated
const unrelated = db.other; // unrelated
offTarget(`SELECT * FROM ${process.env.SCHEMA}.users`); // unrelated exact import scope

import * as sub from "@example/db/other";
sub.query("SELECT * FROM public.users"); // clear: historical namespace subpath selection

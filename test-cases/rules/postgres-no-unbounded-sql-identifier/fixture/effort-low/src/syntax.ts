import { query } from "@example/db";
let mutableStatement = "SELECT * FROM public.users";
query(mutableStatement); // unsupported: mutable declaration
const { text } = { text: "SELECT * FROM public.users" };
query(text); // unsupported: destructured declaration
const alias = mutableStatement;
query(alias); // unsupported: alias initializer
query(); // unsupported: absent argument
query("not SQL !?"); // unsupported: invalid SQL text
query(`SELECT * FROM ${"public"}.users`); // unsupported: relation expression has no finite-source proof
query(`SELECT * FROM ${process.env.SCHEMA}_suffix.users`); // unsupported: composite relation token
(() => query)("SELECT * FROM public.users"); // unsupported: executor reference escape

import { query } from "@example/db";
let a;
const b = a;
a = b;
query(`SELECT id FROM ${b}.users`);

import * as db from "@example/db";
// Maximum effort selects only query members; configured read selects named imports.
db.read(`SELECT * FROM ${process.env.SCHEMA}.users`);
const alias = db;
alias.query(`SELECT * FROM ${process.env.SCHEMA}.users`);

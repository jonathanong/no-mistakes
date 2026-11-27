import pg = require("pg");

new pg["Pool" as const]();

const { Pool: ReqPool } = (require as NodeRequire)("pg");
new ReqPool();

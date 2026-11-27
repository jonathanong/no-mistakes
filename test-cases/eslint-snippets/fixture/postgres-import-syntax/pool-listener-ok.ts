import pg = require("pg");

const p1 = new pg.Pool();
p1["on" as const]("error", console.error);

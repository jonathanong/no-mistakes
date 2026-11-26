declare const flag: string;
const db = require(flag);

db.read("BEGIN");
db.read("SELECT 1");
db.read(flag);

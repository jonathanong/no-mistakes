declare const flag: string;
declare const k: string;
const COLS = { id: "id" };
const db = require(flag);

db.read(`SELECT ${COLS[k]}`);
db.read("SELECT 1");

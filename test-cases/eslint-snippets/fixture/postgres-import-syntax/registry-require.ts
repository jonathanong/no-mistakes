const COLS = { id: "id" };
const db = require("@example/db");
const { read: r } = require("@example/db");
import e = require("@example/db");
declare const k: string;

db.read(`SELECT ${COLS[k]}`);
r(`SELECT ${COLS[k]}`);
e.read(`SELECT ${COLS[k]}`);

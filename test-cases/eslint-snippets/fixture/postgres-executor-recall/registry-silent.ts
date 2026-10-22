import { read } from "@example/db";
import * as db from "@example/db";
import * as other from "@other/db";
const TABLES = { orders: "orders" };
const key = "read";
let mutable = read;
const { [key]: computed } = db;
const { read: defaulted = read } = db;
const { read: foreign } = other;
function shadow(read) { const alias = read; alias(`SELECT ${TABLES[kind]}`); }
function defaults({ read: selected } = db) { selected(`SELECT ${TABLES[kind]}`); }
mutable(`SELECT ${TABLES[kind]}`);
computed(`SELECT ${TABLES[kind]}`);
defaulted(`SELECT ${TABLES[kind]}`);
foreign(`SELECT ${TABLES[kind]}`);
read.apply(null, args);
read.call(...args);
const forward = (sql) => read(sql);
const second = (sql) => forward(sql);
second(`SELECT ${TABLES[kind]}`);

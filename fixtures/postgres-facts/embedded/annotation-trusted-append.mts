import sql, { default as alternate } from "sql-template-strings";
import unknownTag from "@example/unknown-tag";
import { read } from "@bench/db";

const bound = sql`/* bound */`.append(sql`SELECT id FROM users WHERE owner = ${ownerId}`);
read(bound);

// A primitive append is arbitrary code, even when the shared SQL guess is composed.
String.prototype.append = function () { return "SELECT 9"; };
const primitive = "/* misleading */".append("SELECT 1");
read(primitive);

const raw = String.raw`/* misleading */`.append("SELECT 2");
read(raw);

const opaque = unknownTag`/* misleading */`.append("SELECT 3");
read(opaque);

const escaped = sql`/* escaped */`.append(sql`SELECT 4`);
change(escaped);
read(escaped);

const captured = sql`/* captured */`.append(sql`SELECT 5`);
function changeCaptured() { captured.append(sql`SELECT 6`); }
read(captured);

const renamed = alternate`/* renamed */`.append(sql`SELECT ${value}`);
read(renamed);

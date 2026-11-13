import sql from "sql-template-strings";
import { write } from "@app/db";
write(sql`SELECT ${1}`); // tag-scalar-bind
write(sql`SELECT ${{ id: 1 }}`); // tag-record-bind
write(String.raw`SELECT ${2}`); // raw-number
write(String.raw`SELECT\n3`); // raw-source-text
write(String.raw`SELECT ${true}`); // raw-boolean
write(sql`${sql`SELECT 4`}`); // tag-leading-builder
write(sql`SELECT ${sql`fragment`}`); // tag-trailing-builder
function number() { return 5; }
write(sql`SELECT ${number()}`); // tag-call-effect
write(String.raw`SELECT ${{ id: 1 }}`); // raw-record-coercion
function shadow(sql) { return sql`SELECT 6`; }
write(shadow(sql)); // tag-shadowed-binding
function shadowRaw(String) { return String.raw`SELECT 7`; }
write(shadowRaw({ raw: unknownTag })); // raw-shadowed-binding
unknownTag`${() => write("SELECT 8")}`; // tag-opaque-callback
const statement = sql`SELECT 9`;
write(statement, proxy.option); // executor-later-getter

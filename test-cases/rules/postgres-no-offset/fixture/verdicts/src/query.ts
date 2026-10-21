declare const db: { query(sql: string): void };
declare const opaque: string;
declare const choose: boolean;
declare const offset: number;
import sql from "sql-template-strings";

db.query("SELECT id FROM widgets");
db.query(choose ? "SELECT id FROM widgets OFFSET 2" : "SELECT id FROM widgets");
// The clause is fixed before the unknown bind value, so it remains a finding.
db.query(`SELECT id FROM widgets OFFSET ${offset}`);
// A builder's static leading statement proves OFFSET before its opaque append.
db.query(sql`SELECT id FROM widgets OFFSET 4`.append(opaque));
// Concatenation does not have a prepared SQL prefix; abstain on this call.
db.query("SELECT id FROM widgets OFFSET 3 " + opaque);
db.query("SELECT id FROM widgets " + opaque);
db.query(opaque);
db.query("SELECT ???");
// Equal SQL text from distinct branch origins remains an exhaustive union.
db.query(choose ? "SELECT id FROM widgets OFFSET 5" : "SELECT id FROM widgets OFFSET 5");
// A finite known arm plus an opaque arm is Possible, never exhaustive.
db.query(choose ? "SELECT id FROM widgets" : opaque);

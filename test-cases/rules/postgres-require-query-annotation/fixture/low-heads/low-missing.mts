import { read } from "@bench/db";

read("SELECT 1"); // definite direct string
read(`SELECT ${value}`); // definite direct template
read(sql`SELECT ${value}`); // definite tag head; low effort ignores tag identity
const missing = "SELECT 1";
read(missing); // definite one-local-const
read("/* */ /* later name */ SELECT 1"); // first empty comment cannot be repaired
read("BEGIN"); // exact transaction is exempt
read("COMMIT;"); // exact transaction is exempt
read(`SELECT ${value}`); // known non-transaction head despite dynamic tail
read(`B${value}`); // unresolved partial transaction keyword
read(`BE${value}`); // unresolved partial transaction keyword
read(`BEGIN${value}`); // unresolved transaction boundary
read(`/*${value}`); // unresolved partial comment
read(choose ? "SELECT 1" : "SELECT 2"); // unresolved conditional
read(sql`SELECT 1`.append(suffix)); // fluent append remains unresolved
read(buildSql()); // opaque helper remains unresolved
read("SELECT " + suffix); // definite settled SQL head
const combined = "SELECT " + suffix;
read(combined); // definite settled SQL head through one local const
read("B" + suffix); // unresolved partial transaction keyword
read("BE" + suffix); // unresolved partial transaction keyword
read("BEGIN" + suffix); // unresolved transaction boundary
read("/* */" + suffix); // unresolved empty annotation prefix
read(sql`BEGIN`); // fully known tagged transaction remains exempt
read(sql`BEGINNING`); // full literal is missing despite the keyword prefix
const taggedMissing = sql`SELECT ${value}`;
read(taggedMissing); // definite missing head through a safe local const
const escapedMissing = sql`SELECT ${value}`;
stash(escapedMissing);
read(escapedMissing); // escaping the tag value removes the local proof
read(""); // known empty input cannot become an annotation
read(" "); // known whitespace-only input cannot become an annotation
read("/"); // complete literal slash is missing an annotation
read("/*"); // complete unterminated comment is missing an annotation
read("/* unterminated"); // complete unterminated comment is missing an annotation
read("B"); // complete partial transaction keyword is not dynamically completed
read("BE"); // complete partial transaction keyword is not dynamically completed
read(`/${value}`); // a dynamic tail could complete a named comment
read(`/* */${value}`); // a dynamic tail could begin a transaction command
read(`/* */ SELECT ${value}`); // settled SQL after an empty comment is missing
read(`/* */ SELECT ` + suffix); // settled SQL after an empty comment stays missing
read(`BEGINNING${value}`); // settled word boundary cannot become a transaction
read(`BEGIN;${value}`); // settled transaction boundary remains exempt
read(`/* */ BEGIN;${value}`); // optional empty comment does not change transaction exemption
read("BEGINé"); // Unicode word continuation is not a transaction boundary
read("BEGIN。"); // Unicode punctuation is a transaction boundary
read(`/* */ /* later */ BEGIN;${value}`); // fixed boundary uses max's comment semantics
read("/* */ /* later */ BEGIN;"); // same exemption without an unknown suffix

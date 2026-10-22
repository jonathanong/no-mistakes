import { read } from "@bench/db";
import { sql } from "@bench/tags";
const text = "/* scalar */ SELECT 1";
const tagged = sql`/* tagged const */ SELECT 2`;
read(text); // low: true
read(sql`/* direct tag */ SELECT 3`); // low: true
read(tagged); // low: true
read("/* literal */ SELECT 4"); // low: true
// Tests inject diagnostics outside and across ownership references.

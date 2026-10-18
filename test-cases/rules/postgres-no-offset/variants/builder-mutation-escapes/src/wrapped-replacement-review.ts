import sql from "sql-template-strings";
import { read } from "@example/db";

// Type wrappers around a name write do not expose its previous object.
const original = sql`SELECT id FROM accounts WHERE active IS TRUE OFFSET 5`; // expect: offset
// Mutating a separate object's private storage does not mutate this builder.
class Metadata {
  #value = "";
  update() { this.#value = "metadata"; }
}
new Metadata().update();
let q = original;
(q as any) = sql`SELECT id FROM accounts WHERE active IS TRUE`;
read(original);

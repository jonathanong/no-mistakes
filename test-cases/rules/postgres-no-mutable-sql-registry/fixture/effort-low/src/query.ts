import { query } from "@example/db";
import { tables } from "./tables";
import "./mutate";
export function list() {
query("SELECT id FROM app.users"); // EXPECT_CLEAR
query(`SELECT id FROM app.users`); // EXPECT_CLEAR
const literal = "SELECT id FROM app.users";
query(literal); // EXPECT_CLEAR
// Cross-file registry immutability is deliberately not inferred at low effort.
query(`SELECT id FROM ${tables.users}`); // EXPECT_UNSUPPORTED
const indirect = () => "SELECT id FROM app.users";
query(indirect()); // EXPECT_UNSUPPORTED
// no-mistakes-disable-next-line postgres-no-mutable-sql-registry
query(`SELECT id FROM ${tables.users}`); // EXPECT_SUPPRESSED
}

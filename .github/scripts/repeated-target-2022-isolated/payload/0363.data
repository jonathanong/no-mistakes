import { query } from "@example/db";
const yes = true;
const no = false;
const nonempty = "ready";
const empty = "";
const mixed = runtimeFlag ? true : false;
// A recoverable condition never makes an opaque reachable SQL arm safe.
query(yes ? unknownSql : "SELECT 1");
query(no ? "SELECT 2" : unknownSql);
query(nonempty ? unknownSql : "SELECT 3");
query(empty ? "SELECT 4" : unknownSql);
query(mixed ? "SELECT 5" : unknownSql);

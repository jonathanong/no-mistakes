import { read } from "@example/db";
import { read as subRead } from "@example/db/sub";
import { read as otherRead } from "@example/other";
import { exactSource, subSource, otherSource } from "./union-helper";

function exact(query: typeof read) { query("SELECT 1"); }
function sub(query: typeof subRead) { query("SELECT 1"); }
function other(query: typeof otherRead) { query("SELECT 1"); }

exact(read);
sub(read);
other(read);
exactSource(read);
subSource(read);
otherSource(read);

import { read } from "@bench/db";
import otherSql from "@example/other-sql";

// An untrusted tag cannot authorize the raw extractor's static append guess.
// The helper-prefix evaluator must remain authoritative.

read(otherSql`/* stale static guess */`.append(" SELECT from the original argument"));

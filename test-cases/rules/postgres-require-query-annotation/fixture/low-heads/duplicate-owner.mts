import { read } from "@bench/db";
// Keep one lexical owner so the mutated semantic facts affect every named use.
const named = "/* duplicate owner */ SELECT 1";
read(named);
read(named);

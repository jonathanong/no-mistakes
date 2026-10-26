import { read } from "@example/db";
// Identifier substrings do not trigger the loop prefilter.
const before = "meanwhile $for _while";
export function query() { read("SELECT 1"); }

import { read } from "@example/db";
// The substring format triggers only the negative source filter, never a loop proof.
export function query() { read("SELECT 1"); }

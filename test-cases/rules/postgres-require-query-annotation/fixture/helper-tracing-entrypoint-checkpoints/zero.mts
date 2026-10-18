import { read } from "@app/db";
// Top-level executors are excluded from speculative unmodeled entrypoints.
export const statement = "/* zero */ SELECT 1";
read(statement);
read("SELECT 1"); // Known missing annotation; no function entrypoint exists.

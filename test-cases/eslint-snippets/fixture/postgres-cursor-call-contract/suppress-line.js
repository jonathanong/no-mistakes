import { runCursor } from "@db/cursors";
runCursor("SELECT 1"); // no-mistakes-disable-line postgres-cursor-call-contract -- reviewed query

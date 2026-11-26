import { runCursor } from "@db/cursors";
// no-mistakes-disable-next-line postgres-cursor-call-contract -- reviewed query
runCursor("SELECT 1");

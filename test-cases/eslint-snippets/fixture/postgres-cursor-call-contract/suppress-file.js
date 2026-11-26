// no-mistakes-disable-file postgres-cursor-call-contract -- reviewed query
import { runCursor } from "@db/cursors";
runCursor("SELECT 1");

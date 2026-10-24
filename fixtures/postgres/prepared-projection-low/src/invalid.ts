import { query } from "@db/a";
// Invalid SQL must keep the same low-effort abstention and advisory.
query("SELECT FROM invalid_control OFFSET");

// Negative: documented suppressions silence an intentional live scan.
import { readdirSync } from "node:fs";

// no-mistakes-disable-next-line test-no-live-repository-scan
readdirSync("docs");
readdirSync("packages"); // no-mistakes-disable-line test-no-live-repository-scan

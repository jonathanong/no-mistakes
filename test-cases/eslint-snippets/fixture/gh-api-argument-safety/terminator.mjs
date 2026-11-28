import { execFile } from "node:child_process";

execFile("gh", ["api", "repos/O/R/labels", "-f", "x=1", "--", suffix]);
execFile("gh", ["api", "repos/O/R/labels", "-f", "x=1", "--", "--write"]);
execFile("gh", ["api", "repos/O/R/labels", "-X", "POST", "-f", "x=1", "--", suffix]);
execFile("gh", ["api", "--", endpoint, "-f", "x=1"]);

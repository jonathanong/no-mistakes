import cp from "node:child_process";
import { execFile } from "node:child_process";

cp.execFile("gh", ["api", "repos/O/R/labels?per_page=100"]);
const run = execFile;
run("gh", ["api", "repos/O/R/labels?per_page=100"]);
execFile.call(null, "gh", ["api", "repos/O/R/labels?per_page=100"]);
(0, execFile)("gh", ["api", "repos/O/R/labels?per_page=100"]);
cp[`execFile`]("gh", ["api", "repos/O/R/labels?per_page=100"]);
cp?.execFile("gh", ["api", "repos/O/R/labels?per_page=100"]);
const args = ["gh", ["api", "repos/O/R/labels?per_page=100"]];
execFile.apply(null, args);
execFile.apply(null, ["gh", ["api", "repos/O/R/labels?per_page=100"]]);
execFile.call(...["null"]);
// eslint-disable-next-line no-mistakes/gh-api-argument-safety -- retained probe
execFile("gh", ["api", "repos/O/R/labels?per_page=100"]);
// no-mistakes-disable-next-line no-mistakes/gh-api-argument-safety -- retained probe
execFile("gh", ["api", "repos/O/R/labels?per_page=100"]);

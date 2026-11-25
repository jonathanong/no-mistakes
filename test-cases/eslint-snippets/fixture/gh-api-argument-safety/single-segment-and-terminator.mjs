import { execFile } from "node:child_process";
execFile("gh", ["api", "user?per_page=100"]);
execFile("gh", ["api", "graphql?foo=bar"]);
execFile("gh", ["api", `${endpoint}?per_page=100`]);
execFile("gh", ["api", "--", "repos/O/R/labels?per_page=100"]);
execFile("gh", ["api", "-f", "x=1", "--", "repos/O/R/labels"]);
execFile("gh", ["api", "repos/O/R/labels", "-f", "x=1", "--", "-X", "GET"]);
execFile("gh", ["api", "-X=GET", "-f", "x=1", "repos/O/R/labels"]);
execFile("gh", ["api", "-X=", "-f", "x=1", "repos/O/R/labels"]);
execFile("gh", ["api", "--", "-f", "x=1"]);
execFile("gh", ["api", "graphql", "-f", "query=query { viewer { login } }"]);
// no-mistakes-disable-next-line gh-api-argument-safety -- reviewed single-segment query
execFile("gh", ["api", "notifications?all=true"]);

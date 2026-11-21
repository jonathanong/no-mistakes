import { execSync } from "node:child_process";

execSync("cd ../widgets && widget-audit check && cd /repo && widget-audit check");

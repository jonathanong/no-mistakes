import { execSync } from "node:child_process";

execSync("widget-audit check --label 'nightly'");
execSync('widget-audit check --label "nightly"');
execSync('ITEM="placeholder widget-audit check extra"');
execSync('"widget-audit" check');

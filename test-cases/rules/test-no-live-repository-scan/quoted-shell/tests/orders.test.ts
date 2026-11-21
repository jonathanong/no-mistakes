import { execSync } from "node:child_process";

execSync("widget-audit check --label 'nightly'");
execSync('widget-audit check --label "nightly"');
execSync('ITEM="placeholder widget-audit check extra"');
execSync("ITEM='$item' widget-audit check");
execSync("'ITEM=$item' widget-audit check");
execSync('"widget-audit" check');
execSync("widget-audit check --label '$item'");
execSync("widget-audit check --label '`item`'");
execSync('widget-audit check --label "$item"');
execSync('widget-audit check --label "`item`"');
execSync('cd widgets "&&" widget-audit check');
execSync('cd "widgets" && widget-audit check');
execSync('cd "&&" && widget-audit check');

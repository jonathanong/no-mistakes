import { execSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const root = mkdtempSync(join(tmpdir(), "widgets-"));

execSync("cd fixtures && widget-audit check || cd /repo && widget-audit check", { cwd: root });
execSync("cd /repo || cd fixtures && widget-audit check", { cwd: root });
execSync("cd /repo && cd fixtures || widget-audit check", { cwd: root });
execSync("cd /repo || cd fixtures || widget-audit check", { cwd: root });
execSync("cd /repo || cd /workspace && widget-audit check", { cwd: root });
execSync("cd ../widgets || cd /repo && widget-audit check", { cwd: root });
execSync("cd /repo && check-preflight; widget-audit check", { cwd: root });
execSync("cd fixtures || widget-audit check");
execSync("cd /repo || READY=1 && widget-audit check", { cwd: root });

const ready = true;

exec("node scripts/root.mts");
exec("npm run scripts/root.mts");
exec("node missing.mts");
execFile("scripts/exec-file.mts");
fork("scripts/fork.mts");
spawn("scripts/spawn.mts", [], { cwd: "apps/site" });

const config = {
  webServer: { command: "bun scripts/web.mts", cwd: "apps/site" },
};

{
  exec("tsx scripts/block.mts");
}

function declared() {
  return exec("node scripts/function.mts");
}

export const exported = exec("node scripts/export-var.mts");

export function exportedFunction() {
  exec("node scripts/export-function.mts");
}

export default () => {
  exec("node scripts/default-arrow.mts");
};

if (ready) {
  exec("node scripts/if.mts");
} else {
  exec("node scripts/else.mts");
}

try {
  exec("node scripts/try.mts");
} catch (error) {
  exec("node scripts/catch.mts");
} finally {
  exec("node scripts/finally.mts");
}

while (ready) {
  exec("node scripts/while.mts");
  break;
}

for (let i = 0; i < 1; i++) {
  exec("node scripts/for.mts");
}

for (const key in items) {
  exec("node scripts/for-in.mts");
}

for (const item of items) {
  exec("node scripts/for-of.mts");
}

const asyncRunner = async () => {
  await exec("node scripts/await.mts");
};

const nested = wrap(exec("node scripts/nested.mts"));


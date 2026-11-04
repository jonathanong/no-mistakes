const assert = require("node:assert/strict");
const { spawn, spawnSync } = require("node:child_process");
const path = require("node:path");
const test = require("node:test");

const script = path.join(__dirname, "with-resource-samples.sh");
const env = { ...process.env, RESOURCE_SAMPLE_INTERVAL_SECONDS: "0.2" };

test("requires a command", () => {
  const result = spawnSync("bash", [script], { encoding: "utf8", env });

  assert.equal(result.status, 2);
  assert.match(result.stderr, /Usage:/);
});

test("preserves the command's output and exit status", () => {
  const result = spawnSync("bash", [script, "bash", "-c", "echo built; exit 7"], {
    encoding: "utf8",
    env,
  });

  assert.equal(result.status, 7, result.stderr);
  assert.match(result.stdout, /^built$/m);
  assert.match(
    result.stdout,
    /\[resources\] \S+ start mem_available_mib=\S+ swap_free_mib=\S+ disk_available_mib=\S+ oom_kills=\S+ memory_full_avg10=\S+ top_rss=\S+/,
  );
  assert.match(result.stdout, /\[resources\] \S+ exit-7 /);
});

test("samples while the command runs", () => {
  const result = spawnSync("bash", [script, "sleep", "0.7"], { encoding: "utf8", env });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /\[resources\] \S+ running /);
});

test("logs a final sample and forwards SIGTERM to the command", async () => {
  const child = spawn("bash", [script, "sleep", "30"], { env, stdio: ["ignore", "pipe", "pipe"] });
  let stdout = "";
  child.stdout.setEncoding("utf8");
  child.stdout.on("data", (chunk) => {
    stdout += chunk;
    if (stdout.includes(" running ")) {
      child.kill("SIGTERM");
    }
  });

  const status = await new Promise((resolve) => child.on("exit", (code) => resolve(code)));

  assert.equal(status, 143);
  assert.match(stdout, /\[resources\] \S+ received-sigterm /);
});

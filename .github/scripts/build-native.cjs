const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const directory = path.resolve("target/native-build-diagnostics");

function probe(command, args) {
  const result = spawnSync(command, args, { encoding: "utf8", timeout: 10000 });
  return {
    status: result.status,
    error: result.error?.message,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

function metadata() {
  const keys = [
    "RUNNER_OS",
    "RUNNER_ARCH",
    "ImageOS",
    "ImageVersion",
    "GITHUB_SHA",
    "GITHUB_RUN_ID",
    "GITHUB_RUN_ATTEMPT",
    "NATIVE_RUST_CACHE_HIT",
    "RUSTFLAGS",
    "CARGO_BUILD_JOBS",
    "CARGO_INCREMENTAL",
    "CARGO_TARGET_DIR",
    "CARGO_PROFILE_DEV_DEBUG",
    "CARGO_PROFILE_TEST_DEBUG",
    "NO_MISTAKES_BUILD_NAPI",
  ];
  const environment = Object.fromEntries(keys.map((key) => [key, process.env[key] ?? null]));
  return {
    startedAt: new Date().toISOString(),
    platform: process.platform,
    arch: process.arch,
    cpus: os.availableParallelism(),
    totalMemory: os.totalmem(),
    environment,
    cpuModel: os.cpus()[0]?.model,
    release: os.release(),
    rustc: probe("rustc", ["--version", "--verbose"]),
    cargo: probe("cargo", ["--version"]),
    linker: process.platform === "darwin" ? probe("ld", ["-v"]) : null,
    appleTools:
      process.platform === "darwin"
        ? {
            system: probe("sw_vers", []),
            xcode: probe("xcodebuild", ["-version"]),
            clang: probe("clang", ["--version"]),
          }
        : null,
    profiles: fs
      .readFileSync("Cargo.toml", "utf8")
      .split("\n")
      .filter((line) => /^\[profile\.|^(debug|opt-level|codegen-units|lto)\s*=/.test(line)),
    command: ["cargo", "build", "--locked", "-p", "no-mistakes", "--timings", "--verbose"],
  };
}

function sample() {
  const result =
    process.platform === "win32"
      ? probe("powershell", [
          "-NoProfile",
          "-Command",
          'Get-Process | Where-Object { $_.ProcessName -match "^(cargo|rustc|rust-lld|lld|link|clang|cc|ld)$" } | Select-Object Id,ProcessName,CPU,WorkingSet64 | ConvertTo-Json -Compress',
        ])
      : probe("ps", ["-axo", "pid=,ppid=,pcpu=,rss=,etime=,comm="]);
  const output =
    process.platform === "win32"
      ? result.stdout
      : result.stdout
          ?.split("\n")
          .filter((line) => /(?:^|[ /])(cargo|rustc|rust-lld|lld|ld|clang|cc)(?:\s|$)/.test(line))
          .join("\n");
  fs.appendFileSync(
    path.join(directory, "processes.jsonl"),
    `${JSON.stringify({
      at: new Date().toISOString(),
      status: result.status,
      error: result.error?.message,
      output,
      stderr: result.stderr,
      freeMemory: os.freemem(),
      loadAverage: os.loadavg(),
      swap: process.platform === "darwin" ? probe("sysctl", ["vm.swapusage"]) : null,
    })}\n`,
  );
}

function diagnostic(action) {
  try {
    action();
  } catch (error) {
    console.warn(`Native build diagnostics: ${error.message}`);
  }
}

function main() {
  fs.mkdirSync(directory, { recursive: true });
  diagnostic(() =>
    fs.writeFileSync(
      path.join(directory, "metadata.json"),
      `${JSON.stringify(metadata(), null, 2)}\n`,
    ),
  );
  const args = ["build", "--locked", "-p", "no-mistakes", "--timings", "--verbose"];
  const timed = process.platform === "darwin";
  const command = timed ? "/usr/bin/time" : "cargo";
  // Normalize Cargo signals before BSD time handles them; see docs/development/ci-builds.md.
  const commandArgs = timed
    ? [
        "-l",
        "-o",
        path.join(directory, "time.txt"),
        "/bin/sh",
        "-c",
        'cargo "$@"; status=$?; exit "$status"',
        "native-build",
        ...args,
      ]
    : args;
  const log = fs.createWriteStream(path.join(directory, "cargo.log"));
  const started = Date.now();
  const child = spawn(command, commandArgs, {
    stdio: ["inherit", "pipe", "pipe"],
    detached: process.platform !== "win32",
  });
  for (const [source, destination] of [
    [child.stdout, process.stdout],
    [child.stderr, process.stderr],
  ]) {
    source.on("data", (data) => {
      destination.write(data);
      log.write(data);
    });
  }
  diagnostic(sample);
  const interval = setInterval(() => diagnostic(sample), 30000);
  let cancelledSignal;
  function cancel(signal) {
    cancelledSignal = signal;
    diagnostic(() =>
      fs.writeFileSync(
        path.join(directory, "cancellation.json"),
        `${JSON.stringify({
          at: new Date().toISOString(),
          signal,
        })}\n`,
      ),
    );
    if (!child.pid) return;
    if (process.platform === "win32") {
      spawnSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], { timeout: 10000 });
    } else {
      try {
        process.kill(-child.pid, signal);
      } catch (error) {
        if (error.code !== "ESRCH") throw error;
      }
    }
  }
  const interrupt = () => cancel("SIGINT");
  const terminate = () => cancel("SIGTERM");
  process.on("SIGINT", interrupt);
  process.on("SIGTERM", terminate);
  child.on("error", (error) => {
    console.error(error.message);
    diagnostic(() =>
      fs.writeFileSync(path.join(directory, "spawn-error.txt"), `${error.message}\n`),
    );
  });
  child.on("close", (code, signal) => {
    clearInterval(interval);
    process.off("SIGINT", interrupt);
    process.off("SIGTERM", terminate);
    diagnostic(sample);
    diagnostic(() =>
      fs.writeFileSync(
        path.join(directory, "result.json"),
        `${JSON.stringify(
          {
            completedAt: new Date().toISOString(),
            elapsedSeconds: (Date.now() - started) / 1000,
            exitCode: code,
            signal,
            cancelledSignal,
          },
          null,
          2,
        )}\n`,
      ),
    );
    diagnostic(() => {
      const timings = path.resolve(process.env.CARGO_TARGET_DIR || "target", "cargo-timings");
      if (fs.existsSync(timings))
        fs.cpSync(timings, path.join(directory, "cargo-timings"), { recursive: true });
    });
    log.end(() => {
      const termination = cancelledSignal || signal;
      process.exitCode = termination
        ? 128 + (os.constants.signals[termination] || 1)
        : code >= 0
          ? code
          : 1;
    });
  });
}

if (require.main === module) main();
module.exports = { main, metadata, sample, probe };

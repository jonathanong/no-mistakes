const { spawnSync } = require("node:child_process");

function probe(command, args, timeout = 10000) {
  const result = spawnSync(command, args, { encoding: "utf8", timeout, maxBuffer: 1024 * 1024 });
  return {
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

function darwinDiagnostics(run = probe) {
  const columns = "pid=,ppid=,pcpu=,rss=,etime=,state=,comm=";
  return {
    virtualMemory: run("vm_stat", [], 1000),
    memoryPressure: run("memory_pressure", ["-Q"], 1000),
    processes: run("ps", ["-axo", columns], 1000),
    threads: run("ps", ["-M", "-axo", columns], 1000),
  };
}

module.exports = { probe, darwinDiagnostics };

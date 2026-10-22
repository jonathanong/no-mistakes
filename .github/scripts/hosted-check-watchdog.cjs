const { spawn, spawnSync } = require("node:child_process");
const { writeFileSync } = require("node:fs");
const { command } = require("./hosted-check-runtime.cjs");

// A healthy check finishes before this point. Capture a stalled command before
// timeout terminates its process tree and erases the useful evidence.
const SNAPSHOT_BEFORE_DEADLINE_SECONDS = 60;

function processTree(psOutput, rootPid, nativePath) {
  const rows = new Map();
  for (const line of psOutput.split("\n")) {
    const match = line.trim().match(/^(\d+)\s+(\d+)\s+/);
    if (match) rows.set(Number(match[1]), { parent: Number(match[2]), line: line.trim() });
  }
  const selected = new Set([rootPid]);
  // pnpm may put its child in another group and exit before the snapshot.
  if (nativePath)
    for (const [pid, row] of rows)
      if (row.line.split(/\s+/).includes(nativePath)) selected.add(pid);
  let changed;
  do {
    changed = false;
    for (const [pid, row] of rows) {
      if (!selected.has(pid) && selected.has(row.parent)) {
        selected.add(pid);
        changed = true;
      }
    }
  } while (changed);
  return (
    [
      "PID PPID PGID STAT ELAPSED %CPU RSS ARGS",
      ...[...selected]
        .sort((a, b) => a - b)
        .flatMap((pid) => (rows.has(pid) ? [rows.get(pid).line] : [])),
    ].join("\n") + "\n"
  );
}

function readLiveProcesses(cwd, psExecutable = "ps") {
  const snapshot = spawnSync(
    psExecutable,
    ["-eo", "pid=,ppid=,pgid=,stat=,etime=,pcpu=,rss=,args="],
    { cwd, encoding: "utf8", timeout: 2000, killSignal: "SIGKILL", maxBuffer: 1024 * 1024 },
  );
  if (snapshot.error || snapshot.status !== 0)
    throw new Error(`ps failed: ${snapshot.error || snapshot.stderr}`);
  return snapshot.stdout;
}

function savePostExitSnapshot(prefix, cwd) {
  try {
    writeFileSync(
      `${prefix}.processes.txt`,
      command("ps", ["-eo", "pid,ppid,pgid,etime,pcpu,rss,comm"], cwd),
    );
  } catch (error) {
    writeFileSync(`${prefix}.processes.txt`, String(error));
  }
}

function timedCommand(
  executable,
  args,
  {
    cwd,
    stdio,
    snapshotPath,
    nativePath,
    deadlineSeconds,
    snapshotDelayMs,
    readProcessTable = readLiveProcesses,
  } = {},
) {
  const delay = snapshotDelayMs ?? (deadlineSeconds - SNAPSHOT_BEFORE_DEADLINE_SECONDS) * 1000;
  if (!Number.isFinite(delay) || delay <= 0) throw new Error("Invalid live snapshot delay");
  const child = spawn(executable, args, { cwd, stdio });
  const timer = setTimeout(() => {
    if (child.exitCode !== null || child.signalCode !== null) return;
    try {
      writeFileSync(snapshotPath, processTree(readProcessTable(cwd), child.pid, nativePath));
    } catch (error) {
      try {
        writeFileSync(snapshotPath, `Live process snapshot failed: ${error}\n`);
      } catch {
        // Diagnostic I/O must not replace the measured command's exit status.
      }
    }
  }, delay);
  return new Promise((resolve, reject) => {
    child.once("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.once("close", (status, signal) => {
      clearTimeout(timer);
      resolve({ status, signal });
    });
  });
}

module.exports = {
  processTree,
  readLiveProcesses,
  savePostExitSnapshot,
  timedCommand,
  SNAPSHOT_BEFORE_DEADLINE_SECONDS,
};

import { readFileSync, readdirSync } from "node:fs";

function processRecord(pid) {
  const stat = readFileSync(`/proc/${pid}/stat`, "utf8");
  const fields = stat
    .slice(stat.lastIndexOf(")") + 2)
    .trim()
    .split(/\s+/);
  return {
    pid: Number(pid),
    comm: stat.slice(stat.indexOf("(") + 1, stat.lastIndexOf(")")),
    state: fields[0],
    group: Number(fields[2]),
    session: Number(fields[3]),
    userTicks: Number(fields[11]),
    systemTicks: Number(fields[12]),
    start: fields[19],
    rssPages: Number(fields[21]),
  };
}

export function createSdkChildDiagnostics(child) {
  const started = performance.now();
  const lifecycle = {
    startedAt: new Date().toISOString(),
    events: [],
    streams: {},
    diagnosticErrors: [],
  };
  let identity;
  const stamp = () => ({ at: new Date().toISOString(), elapsedMs: performance.now() - started });
  function safely(action) {
    try {
      return action();
    } catch (error) {
      lifecycle.diagnosticErrors.push(String(error?.message ?? error));
    }
  }
  function record(phase, details = {}) {
    safely(() => lifecycle.events.push({ ...stamp(), phase, ...details }));
  }
  safely(() => {
    if (child.pid) identity = processRecord(child.pid);
  });
  record("observe-start", { pid: child.pid });
  return {
    lifecycle,
    record,
    output(stream, chunk) {
      safely(() => {
        const previous = lifecycle.streams[stream];
        const current = stamp();
        if (!previous) lifecycle.events.push({ ...current, phase: `${stream}-first` });
        lifecycle.streams[stream] = {
          bytes: (previous?.bytes ?? 0) + Buffer.byteLength(chunk),
          first: previous?.first ?? current,
          last: current,
        };
      });
    },
    snapshot() {
      safely(() => {
        if (!identity || child.exitCode !== null || child.signalCode !== null) return;
        const leader = processRecord(child.pid);
        if (
          leader.start !== identity.start ||
          leader.group !== child.pid ||
          leader.session !== child.pid
        )
          return;
        const candidates = readdirSync("/proc")
          .filter((pid) => /^\d+$/.test(pid))
          .sort((a, b) => Number(a) - Number(b));
        const processes = [];
        const scanDeadline = performance.now() + 50;
        let scanned = 0;
        for (const pid of candidates.slice(0, 4096)) {
          if (performance.now() >= scanDeadline) break;
          scanned++;
          try {
            const entry = processRecord(pid);
            if (entry.group === child.pid) processes.push(entry);
          } catch (error) {
            if (error.code !== "ENOENT" && error.code !== "ESRCH")
              lifecycle.diagnosticErrors.push(String(error.message));
          }
        }
        record("watchdog-processes", {
          scanned,
          truncated: scanned < candidates.length,
          processes,
        });
      });
    },
    finish() {
      safely(() => {
        for (const [stream, details] of Object.entries(lifecycle.streams))
          lifecycle.events.push({ ...details.last, phase: `${stream}-last`, bytes: details.bytes });
        lifecycle.events.sort((a, b) => a.elapsedMs - b.elapsedMs);
        lifecycle.elapsedMs = performance.now() - started;
      });
    },
  };
}

'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const assert = require('node:assert/strict');
const { pathToFileURL } = require('node:url');
const sha = f => crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex');
async function main() {
  const c = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
  assert.equal(c.status, 'REVIEWED_WORKER_INVOCATION');
  assert.equal(sha(c.addon), c.addon_sha256);
  assert.equal(sha(c.facade), c.facade_sha256);
  const request = JSON.parse(fs.readFileSync(c.request, 'utf8'));
  assert.equal(sha(c.request), c.request_sha256);
  request.timeout = 0; request.lockTimeout = 0;
  delete process.env.RAYON_NUM_THREADS;
  if (c.jobs !== 'default') request.jobs = Number(c.jobs);
  if (c.diagnostics === 'on') request.diagnosticsId = `api-profile-${process.pid}`;
  process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH = path.resolve(c.addon);
  const native = require(path.resolve(c.addon));
  const api = c.mode === 'facade-esm' ? await import(pathToFileURL(path.resolve(c.facade))) : require(path.resolve(c.facade));
  const snapshots = []; let sampling = false;
  async function snapshot() {
    if (sampling) return;
    sampling = true;
    try {
      const raw = await native.diagnosticsSnapshotJson(Buffer.from(JSON.stringify({ id: request.diagnosticsId })));
      snapshots.push({ monotonic_ns: process.hrtime.bigint().toString(), value: JSON.parse(raw) });
    } finally { sampling = false; }
  }
  const cpu0 = process.cpuUsage(), start = process.hrtime.bigint();
  const pending = c.mode === 'raw-native' ? native.analyzeProjectJson(Buffer.from(JSON.stringify(request))) : api.analyzeProject(request);
  assert(pending instanceof Promise);
  const timer = c.diagnostics === 'on' ? setInterval(() => { snapshot().catch(e => { fs.writeFileSync(c.observer_error, String(e)); }); }, 5) : null;
  let result;
  try { result = await pending; } finally { if (timer) clearInterval(timer); }
  const wall = Number(process.hrtime.bigint() - start) / 1e9, cpu = process.cpuUsage(cpu0);
  const inheritedRusagePeakRssAtApiCompletion = process.resourceUsage().maxRSS;
  const apiCompletionProcStatus = fs.readFileSync('/proc/self/status', 'utf8');
  const vmHwm = /^VmHWM:\s+(\d+)\s+kB$/m.exec(apiCompletionProcStatus);
  assert(vmHwm, 'Linux /proc/self/status VmHWM is required');
  const peakRssAtApiCompletion = Number(vmHwm[1]);
  assert(Number.isSafeInteger(peakRssAtApiCompletion) && peakRssAtApiCompletion > 0);
  while (sampling) await new Promise(resolve => setImmediate(resolve));
  assert(!fs.existsSync(c.observer_error));
  const bytes = c.mode === 'raw-native' ? Buffer.from(result) : Buffer.from(JSON.stringify(result));
  const parsed = JSON.parse(bytes); assert(parsed.reports.length > 0);
  assert(parsed.reports.every(r => r.result !== null));
  fs.writeFileSync(c.output, bytes);
  fs.writeFileSync(c.snapshots, JSON.stringify(snapshots));
  fs.writeFileSync(c.receipt, JSON.stringify({ status: 'COMPLETE', mode: c.mode, wall_seconds: wall,
    node_version: process.version, node_versions: process.versions, platform: process.platform, arch: process.arch,
    environment: { UV_THREADPOOL_SIZE: process.env.UV_THREADPOOL_SIZE ?? null, RAYON_NUM_THREADS: process.env.RAYON_NUM_THREADS ?? null, requested_jobs: c.jobs, diagnostics: c.diagnostics },
    cpu_seconds: (cpu.user + cpu.system) / 1e6, user_seconds: cpu.user / 1e6, sys_seconds: cpu.system / 1e6,
    peak_rss_kib: peakRssAtApiCompletion, peak_rss_source: 'linux-proc-self-status-VmHWM-exec-mm', api_completion_proc_status: apiCompletionProcStatus, inherited_rusage_peak_rss_kib: inheritedRusagePeakRssAtApiCompletion, post_verification_peak_rss_kib: process.resourceUsage().maxRSS, output_sha256: sha(c.output), output_bytes: bytes.length,
    reports: parsed.reports.length, snapshot_count: snapshots.filter(s => s.value !== null).length,
    interpretation: 'Whole async API await, including cold request cache construction and serialization; not isolated serializer time. PeakRSS includes fresh Node/addon process.'
  }, null, 2));
}
main().catch(e => { console.error(e); process.exitCode = 1; });

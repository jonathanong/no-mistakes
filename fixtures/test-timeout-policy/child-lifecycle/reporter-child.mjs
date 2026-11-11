import {
  installBrowserLifecycleReceipts,
  recordBrowserLifecycle,
} from '../browser-lifecycle-probe.mjs';

const mode = process.argv[2];
const closeCallbacks = [];
const expected = mode === 'expected-reject';
const reporter = installBrowserLifecycleReceipts({
  onInit() {},
  onBrowserInit() {},
  onTestRunStart() {},
  onTestModuleCollected() {},
  onTestCaseResult() {},
  onTestRunEnd() {},
});

await reporter.onInit({ onClose(callback) { closeCallbacks.push(callback); } });
await reporter.onBrowserInit({ name: 'saved-browser-project' });

if (mode === 'hang-before-body') {
  recordBrowserLifecycle('child-waiting-before-test-body');
  setInterval(() => {}, 1000);
} else {
  await reporter.onTestRunStart([{ id: 'saved-test' }]);
  await reporter.onTestModuleCollected({ moduleId: 'saved-case.test.mjs' });
  await reporter.onTestCaseResult({ name: 'saved browser case', result: { state: expected ? 'failed' : 'passed' } });
  recordBrowserLifecycle('browser-test-body-complete', { expectedReject: expected });
  if (mode === 'hang-after-body') {
    setInterval(() => {}, 1000);
  } else {
    await reporter.onTestRunEnd([], [], expected ? 'failed' : 'passed');
    for (const callback of closeCallbacks) callback();
    process.exitCode = expected ? 1 : 0;
  }
}

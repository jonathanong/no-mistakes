import {
  assertTestTimeout,
  assertTimeoutConfig,
  snapshotTimeoutOptions,
  protectTimeoutConfig,
  guardRegistration,
  guardPublicRunner,
  guardRuntimeConfig,
  guardVitestExports,
  protectRunnerTimeouts,
  type Registration,
  type Kind,
  type RunnerConstructor,
  type Protector,
} from "no-mistakes/test-timeout-policy";
import {
  testTimeoutPolicyPlugin,
  browserTestTimeoutPolicyPlugin,
  type NodeRunnerPolicy,
} from "no-mistakes/test-timeout-policy/plugin";

const options = { timeout: 100, retries: 2, name: "case" };
const snapshot = snapshotTimeoutOptions(options, "test", 300, true);
// Option-specific fields must survive the snapshot's inferred return type.
const retries: number = snapshot.retries;
const name: string = snapshot.name;
const exact: typeof options = snapshot;
const registration: Registration = () => undefined;
const kind: Kind = "hook";
const guardedRegistration: typeof registration = guardRegistration(registration, kind);
class Runner {
  config = {};
  custom = "runner-specific";
}
const constructor: RunnerConstructor = Runner;
const protector: Protector = protectRunnerTimeouts;
const guardedRunner: typeof Runner = guardPublicRunner(Runner, 300, protector);
const runtime: typeof options = guardRuntimeConfig(options);
const facade: typeof options = guardVitestExports(options);
const runnerPolicy: NodeRunnerPolicy = { runner: null };
const timeout: number = assertTestTimeout(100, "test");
assertTimeoutConfig(options, 300);
protectTimeoutConfig(options, 300);
protectRunnerTimeouts(new Runner(), 300);
testTimeoutPolicyPlugin(300, ["vitest"], runnerPolicy);
browserTestTimeoutPolicyPlugin(300, ["vitest"]);
void [
  retries,
  name,
  exact,
  guardedRegistration,
  constructor,
  guardedRunner,
  runtime,
  facade,
  timeout,
];

// A protector may require fields on the constructor's concrete instance type.
const specificProtector: Protector<Runner> = (runner, maximum) => {
  const custom: string = runner.custom;
  void [custom, maximum];
};
guardPublicRunner(Runner, 300, specificProtector);
guardPublicRunner(Runner, 300, (runner: Runner) => {
  void runner.custom;
});
class UnrelatedRunner {
  config = {};
  unrelated = 1;
}
const unrelatedProtector: Protector<UnrelatedRunner> = (runner) => {
  void runner.unrelated;
};
// An unrelated callback must not be accepted for this runner's constructor.
// @ts-expect-error UnrelatedRunner requires fields absent from Runner.
guardPublicRunner(Runner, 300, unrelatedProtector);

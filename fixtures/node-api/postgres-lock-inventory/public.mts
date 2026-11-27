import type {
  LockInventoryBaseline,
  LockInventoryBaselineUnanalyzed,
  LockInventoryReport,
  LockInventoryUnanalyzedDelta,
  PostgresLockInventoryOptions,
  PostgresLockInventoryRuleOptions,
} from "no-mistakes";

const _ruleOptions: PostgresLockInventoryRuleOptions = {
  importSpecifier: "@example/db",
  executorNames: ["query"],
  baselineFile: ".no-mistakes/postgres-locks.json",
  baselineUnanalyzed: true,
  helpers: ["reserveJob"],
};

const _entry: LockInventoryBaselineUnanalyzed = {
  file: "src/orders/repo.ts",
  reason: "executor SQL could not be resolved",
  count: 2,
};

const _baseline: LockInventoryBaseline = {
  version: 1,
  sites: [],
  unanalyzed: [_entry],
};

const _delta: LockInventoryUnanalyzedDelta = {
  file: "src/orders/repo.ts",
  reason: "executor SQL could not be resolved",
  baselineCount: 2,
  currentCount: 3,
};

const _callOptions: PostgresLockInventoryOptions = {
  root: ".",
  config: ".no-mistakes.yml",
  writeBaseline: false,
};

const _report: LockInventoryReport = {
  sites: [],
  counts: {},
  unanalyzed: [],
  unanalyzedByReason: { "executor SQL could not be resolved": 2 },
  unanalyzedByDirectory: { "src/orders": 2 },
  baselineFile: ".no-mistakes/postgres-locks.json",
  baselineUnanalyzed: true,
  added: [],
  removed: [],
  unanalyzedAdded: [_delta],
  unanalyzedRemoved: [],
  baselineUpdated: false,
};

export { _baseline, _callOptions, _delta, _entry, _report, _ruleOptions };

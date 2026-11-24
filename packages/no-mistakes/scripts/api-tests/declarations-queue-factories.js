const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("queue factory configuration selectors are public and exported", () => {
  const types = readFileSync(join(packageRoot, "queue-factory-types.d.ts"), "utf8");
  for (const name of [
    "QueueFactoryArgument",
    "QueueFactoryPropertyArgument",
    "QueueEnqueueFactoryBase",
    "QueueEnqueueFactoryOptions",
    "QueueFactoriesOptions",
    "QueueFactories",
  ]) {
    assert.match(types, new RegExp(`export (?:type|interface) ${name}\\b`));
  }
  assert.match(types, /arg: number;/);
  assert.match(types, /property: string;/);
  assert.match(types, /job: QueueFactoryArgument;/);
  assert.match(types, /queue: QueueFactoryArgument; queueName\?: never/);
  assert.match(types, /queueName: string; queue\?: never/);
  assert.match(types, /defaults\?: QueueFactoryArgument;/);
  assert.match(types, /optionsOnReturnedCall\?: number;/);
  assert.match(types, /options\?: QueueFactoryArgument;/);
  for (const file of ["types.d.ts", "index.d.ts"]) {
    assert.match(
      readFileSync(join(packageRoot, file), "utf8"),
      /export type \* from "\.\/queue-factory-types";/,
    );
  }
});

test("queue policy options and advisory findings have public types", () => {
  const config = readFileSync(join(packageRoot, "queue-policy-types.d.ts"), "utf8");
  assert.match(config, /export type QueueRetentionKey = "removeOnComplete" \| "removeOnFail";/);
  assert.match(
    config,
    /export interface UnboundedJobRetentionConfig \{[\s\S]*?keys\?: QueueRetentionKey\[\];[\s\S]*?allowPaths\?: string\[\];/,
  );
  assert.match(
    config,
    /export interface JobStateAsRecordConfig \{[\s\S]*?allowPaths\?: string\[\];/,
  );
  const report = readFileSync(join(packageRoot, "queue-report-types.d.ts"), "utf8");
  assert.match(
    report,
    /export type QueuePolicyFindingKind = "unbounded-job-retention" \| "queue-job-state-as-record";/,
  );
  assert.match(report, /export type QueueFindingSeverity = "advisory";/);
  assert.match(
    report,
    /export interface QueueCheckFinding \{[\s\S]*?severity\?: QueueFindingSeverity;/,
  );
});
